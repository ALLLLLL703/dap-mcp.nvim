use std::future::Future;
use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use super::schema::{DapVariable, LuaStartArgs, StartDebuggingArgs, VariableChildrenArgs};
use crate::error::SidecarError;
use crate::launch::{LaunchRequest, select_configuration};
use crate::nvim::{BridgeShared, DebugEvent, NvimBridge};

/// Shared debugger operations used by all per-session MCP handlers.
#[derive(Clone)]
pub struct DebugRuntime {
    bridge: NvimBridge,
    shared: BridgeShared,
    timeout: Duration,
    allow_external_files: bool,
}

impl DebugRuntime {
    /// Creates a runtime over the parent-Neovim RPC bridge.
    #[must_use]
    pub const fn new(
        bridge: NvimBridge,
        shared: BridgeShared,
        timeout: Duration,
        allow_external_files: bool,
    ) -> Self {
        Self {
            bridge,
            shared,
            timeout,
            allow_external_files,
        }
    }

    /// Selects a launch.json configuration and waits for initialization.
    pub async fn start(&self, args: StartDebuggingArgs) -> Result<Value, SidecarError> {
        if args
            .test_name
            .as_deref()
            .is_some_and(|name| !name.is_empty())
        {
            return Err(SidecarError::UnsupportedTestTarget);
        }
        let configuration = select_configuration(LaunchRequest {
            file_path: &args.file_full_path,
            working_directory: &args.working_directory,
            configuration_name: &args.configuration_name,
            allow_external_files: self.allow_external_files,
        })?;
        self.call_and_wait(
            "start_debugging",
            &LuaStartArgs { configuration },
            |event| matches!(event, DebugEvent::SessionStarted { .. }),
        )
        .await
    }

    /// Calls a Lua operation and decodes its immediate response.
    pub async fn call<T, R>(&self, method: &str, args: &T) -> Result<R, SidecarError>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.bridge.call(method, args).await
    }

    /// Calls a control action and waits for the matching DAP lifecycle event.
    pub async fn action(
        &self,
        method: &str,
        expected: fn(&DebugEvent) -> bool,
    ) -> Result<Value, SidecarError> {
        self.call_and_wait(method, &json!({}), expected).await
    }

    /// Loads visible variables from the stopped frame.
    pub async fn variables(&self) -> Result<Vec<DapVariable>, SidecarError> {
        self.call("variables", &json!({})).await
    }

    /// Loads direct children of a structured variable.
    pub async fn variable_children(
        &self,
        variables_reference: i64,
    ) -> Result<Vec<DapVariable>, SidecarError> {
        self.call(
            "variable_children",
            &VariableChildrenArgs {
                variables_reference,
            },
        )
        .await
    }

    /// Sends a call after subscribing, then waits without polling Neovim.
    async fn call_and_wait<T, F>(
        &self,
        method: &str,
        args: &T,
        expected: F,
    ) -> Result<Value, SidecarError>
    where
        T: Serialize,
        F: Fn(&DebugEvent) -> bool,
    {
        let mut events = self.shared.events.subscribe();
        let response = self.bridge.call(method, args).await?;
        self.wait_for_event(method, &mut events, expected).await?;
        Ok(response)
    }

    /// Waits for one matching event with the configured operation timeout.
    async fn wait_for_event<F>(
        &self,
        method: &str,
        events: &mut tokio::sync::broadcast::Receiver<DebugEvent>,
        expected: F,
    ) -> Result<(), SidecarError>
    where
        F: Fn(&DebugEvent) -> bool,
    {
        let waiter = async {
            loop {
                if events.recv().await.is_ok_and(|event| expected(&event)) {
                    return;
                }
            }
        };
        tokio::time::timeout(self.timeout, waiter)
            .await
            .map_err(|_| SidecarError::RpcTimeout {
                method: method.to_owned(),
            })
    }

    /// Executes a future under the MCP tool backstop timeout.
    pub async fn with_backstop<F, T>(&self, method: &str, future: F) -> Result<T, SidecarError>
    where
        F: Future<Output = Result<T, SidecarError>>,
    {
        tokio::time::timeout(self.timeout + Duration::from_secs(30), future)
            .await
            .map_err(|_| SidecarError::RpcTimeout {
                method: method.to_owned(),
            })?
    }
}
