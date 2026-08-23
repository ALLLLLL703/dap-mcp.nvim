use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use nvim_rs::{Handler, Neovim};
use rmpv::Value;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock, broadcast, oneshot};
use tokio_util::compat::Compat;
use tokio_util::sync::CancellationToken;

use crate::state::{DebugState, DebugStatus};

/// Writer used by a sidecar connected to its parent Neovim process.
pub type ParentWriter = Compat<tokio::fs::File>;

/// DAP lifecycle event emitted by the Lua plugin.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DebugEvent {
    /// A named debug session was initialized.
    SessionStarted {
        /// Selected launch configuration.
        configuration_name: String,
    },
    /// The debuggee stopped, possibly before a frame became available.
    Stopped {
        /// DAP thread identifier.
        thread_id: Option<i64>,
        /// Current frame when already available.
        frame_id: Option<i64>,
    },
    /// The debuggee continued running.
    Continued,
    /// Debug adapter stdout, stderr, console, or telemetry output.
    Output {
        /// Optional DAP output category.
        category: Option<String>,
        /// Adapter-provided output text.
        output: String,
    },
    /// The debug session terminated or exited.
    Terminated,
}

/// State shared by the Neovim handler, MCP tools, and shutdown path.
#[derive(Clone)]
pub struct BridgeShared {
    /// Pending Lua responses by request identifier.
    pub pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    /// Current debug state.
    pub debug_state: Arc<RwLock<DebugState>>,
    /// Event broadcast used by tool waiters.
    pub events: broadcast::Sender<DebugEvent>,
    /// Sidecar shutdown signal.
    pub shutdown: CancellationToken,
}

impl BridgeShared {
    /// Creates isolated shared bridge state.
    #[must_use]
    pub fn new() -> Self {
        let (events, _) = broadcast::channel(64);
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            debug_state: Arc::new(RwLock::new(DebugState::default())),
            events,
            shutdown: CancellationToken::new(),
        }
    }

    /// Applies one Lua DAP event to the single-session state machine.
    async fn apply_event(&self, event: &DebugEvent) {
        let mut state = self.debug_state.write().await;
        let result = match event {
            DebugEvent::SessionStarted { configuration_name } => {
                state.start(configuration_name.clone())
            }
            DebugEvent::Stopped {
                thread_id,
                frame_id,
            } => state.mark_stopped(*thread_id, *frame_id),
            DebugEvent::Continued if state.status == DebugStatus::Stopped => state.mark_running(),
            DebugEvent::Continued => Ok(()),
            DebugEvent::Output { .. } => Ok(()),
            DebugEvent::Terminated => {
                state.terminate();
                Ok(())
            }
        };
        if let Err(error) = result {
            tracing::warn!(event = "dap.event.invalid", error = %error);
        }
    }
}

impl Default for BridgeShared {
    /// Creates default shared bridge state.
    fn default() -> Self {
        Self::new()
    }
}

/// Handles notifications sent by the parent Neovim instance.
#[derive(Clone)]
pub struct NvimHandler {
    shared: BridgeShared,
}

impl NvimHandler {
    /// Creates a handler backed by shared bridge state.
    #[must_use]
    pub const fn new(shared: BridgeShared) -> Self {
        Self { shared }
    }
}

#[async_trait]
impl Handler for NvimHandler {
    type Writer = ParentWriter;

    /// Routes Lua responses, DAP events, and shutdown notifications.
    async fn handle_notify(&self, name: String, args: Vec<Value>, _neovim: Neovim<Self::Writer>) {
        match name.as_str() {
            "dap_mcp_response" => self.handle_response(args).await,
            "dap_mcp_event" => self.handle_event(args).await,
            "dap_mcp_shutdown" => {
                tracing::info!(event = "rpc.shutdown.received");
                self.shared.shutdown.cancel();
            }
            _ => tracing::debug!(event = "rpc.notification.unknown", name),
        }
    }
}

impl NvimHandler {
    /// Resolves one pending bridge request.
    async fn handle_response(&self, args: Vec<Value>) {
        let Some(request_id) = args.first().and_then(Value::as_u64) else {
            tracing::warn!(event = "rpc.response.invalid_id");
            return;
        };
        let Some(payload) = args.get(1).cloned() else {
            tracing::warn!(event = "rpc.response.missing_payload", request_id);
            return;
        };
        if let Some(sender) = self.shared.pending.lock().await.remove(&request_id) {
            let _ignored = sender.send(payload);
        }
    }

    /// Decodes and broadcasts one DAP lifecycle event.
    async fn handle_event(&self, args: Vec<Value>) {
        let Some(payload) = args.first().cloned() else {
            tracing::warn!(event = "dap.event.missing_payload");
            return;
        };
        let Ok(event) = rmpv::ext::from_value::<DebugEvent>(payload) else {
            tracing::warn!(event = "dap.event.invalid_payload");
            return;
        };
        self.shared.apply_event(&event).await;
        let _receiver_count = self.shared.events.send(event);
    }
}
