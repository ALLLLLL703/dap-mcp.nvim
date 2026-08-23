use tracing_subscriber::EnvFilter;

use crate::config::LogLevel;

/// Initializes structured stderr logging without corrupting Msgpack-RPC stdout.
pub fn initialize(level: LogLevel) {
    let directive = match level {
        LogLevel::Debug => "warn,dap_mcp_sidecar=debug,rmcp=info",
        LogLevel::Info => "warn,dap_mcp_sidecar=info,rmcp=info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(directive))
        .with_writer(std::io::stderr)
        .with_target(false)
        .try_init()
        .ok();
}
