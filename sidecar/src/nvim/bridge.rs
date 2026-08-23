use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use nvim_rs::Neovim;
use rmpv::Value;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::oneshot;

use super::handler::{BridgeShared, ParentWriter};
use crate::error::SidecarError;

/// Successful or failed response emitted by the Lua RPC dispatcher.
#[derive(Debug, serde::Deserialize)]
struct BridgeEnvelope<T> {
    /// Whether the operation succeeded.
    ok: bool,
    /// Successful result.
    result: Option<T>,
    /// Structured failure.
    error: Option<RemoteError>,
}

/// Structured Lua-side failure.
#[derive(Debug, serde::Deserialize)]
struct RemoteError {
    /// Stable error code.
    code: String,
    /// Actionable English message.
    message: String,
}

/// Async request/response client for Lua operations in parent Neovim.
#[derive(Clone)]
pub struct NvimBridge {
    neovim: Neovim<ParentWriter>,
    shared: BridgeShared,
    next_request_id: Arc<AtomicU64>,
    timeout: Duration,
}

impl NvimBridge {
    /// Creates a bridge and registers the parent RPC channel with Lua.
    pub async fn new(
        neovim: Neovim<ParentWriter>,
        shared: BridgeShared,
        timeout: Duration,
    ) -> Result<Self, SidecarError> {
        let bridge = Self {
            neovim,
            shared,
            next_request_id: Arc::new(AtomicU64::new(1)),
            timeout,
        };
        bridge.register_channel().await?;
        Ok(bridge)
    }

    /// Dispatches one typed operation and awaits its Lua notification response.
    pub async fn call<T, R>(&self, method: &str, arguments: &T) -> Result<R, SidecarError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let payload = encode_arguments(arguments)?;
        let (sender, receiver) = oneshot::channel();
        self.shared.pending.lock().await.insert(request_id, sender);
        let dispatch_result = self
            .neovim
            .exec_lua(
                "return require('dap-mcp.rpc').dispatch(...)",
                vec![Value::from(method), Value::from(request_id), payload],
            )
            .await;
        if let Err(error) = dispatch_result {
            self.shared.pending.lock().await.remove(&request_id);
            return Err(SidecarError::RpcCall {
                message: error.to_string(),
            });
        }
        let response = tokio::time::timeout(self.timeout, receiver)
            .await
            .map_err(|_| SidecarError::RpcTimeout {
                method: method.to_owned(),
            })?
            .map_err(|error| SidecarError::RpcCall {
                message: error.to_string(),
            })?;
        let envelope = rmpv::ext::from_value::<BridgeEnvelope<R>>(response).map_err(|error| {
            SidecarError::RpcValue {
                message: error.to_string(),
            }
        })?;
        decode_envelope(envelope)
    }

    /// Retrieves and registers this sidecar's parent RPC channel identifier.
    async fn register_channel(&self) -> Result<(), SidecarError> {
        let info = self
            .neovim
            .get_api_info()
            .await
            .map_err(|error| SidecarError::RpcCall {
                message: error.to_string(),
            })?;
        let channel =
            info.first()
                .and_then(Value::as_u64)
                .ok_or_else(|| SidecarError::RpcValue {
                    message: "nvim_get_api_info did not return a channel id".to_owned(),
                })?;
        self.neovim
            .exec_lua(
                "return require('dap-mcp.rpc').register(...) ",
                vec![Value::from(channel)],
            )
            .await
            .map_err(|error| SidecarError::RpcCall {
                message: error.to_string(),
            })?;
        Ok(())
    }
}

/// Preserves struct field names when converting arguments to Msgpack values.
fn encode_arguments<T: Serialize>(arguments: &T) -> Result<Value, SidecarError> {
    let named = serde_json::to_value(arguments).map_err(|error| SidecarError::RpcValue {
        message: error.to_string(),
    })?;
    rmpv::ext::to_value(named).map_err(|error| SidecarError::RpcValue {
        message: error.to_string(),
    })
}

/// Converts a Lua envelope into a typed result or stable remote error.
fn decode_envelope<T>(envelope: BridgeEnvelope<T>) -> Result<T, SidecarError> {
    if envelope.ok {
        return envelope.result.ok_or_else(|| SidecarError::RpcValue {
            message: "successful bridge response omitted result".to_owned(),
        });
    }
    let error = envelope.error.ok_or_else(|| SidecarError::RpcValue {
        message: "failed bridge response omitted error".to_owned(),
    })?;
    Err(SidecarError::Remote {
        code: error.code,
        message: error.message,
    })
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{BridgeEnvelope, RemoteError, decode_envelope, encode_arguments};

    /// Decodes a successful typed envelope.
    #[test]
    fn decodes_success() {
        let result = decode_envelope(BridgeEnvelope {
            ok: true,
            result: Some(7_u64),
            error: None,
        });
        assert_eq!(result.expect("success value"), 7);
    }

    /// Preserves a remote stable error code.
    #[test]
    fn decodes_remote_error() {
        let result = decode_envelope::<u64>(BridgeEnvelope {
            ok: false,
            result: None,
            error: Some(RemoteError {
                code: "NO_SESSION".to_owned(),
                message: "No active session".to_owned(),
            }),
        });
        let error = result.expect_err("remote failure");
        assert_eq!(error.code(), "NO_SESSION");
    }

    /// Preserves named struct fields for Lua table access.
    #[test]
    fn encodes_struct_arguments_as_named_map() {
        #[derive(Serialize)]
        struct Arguments {
            file_path: String,
        }
        let encoded = encode_arguments(&Arguments {
            file_path: "/tmp/main.rs".to_owned(),
        })
        .expect("encode arguments");
        assert!(encoded.as_map().is_some());
        assert!(encoded.to_string().contains("file_path"));
    }
}
