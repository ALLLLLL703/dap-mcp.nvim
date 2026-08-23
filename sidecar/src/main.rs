//! Executable composition root for the dap-mcp.nvim sidecar.

use std::time::Duration;

use clap::Parser;
use dap_mcp_sidecar::config::{Cli, RuntimeConfig};
use dap_mcp_sidecar::error::SidecarError;
use dap_mcp_sidecar::logging;
use dap_mcp_sidecar::mcp::{DebugRuntime, serve_http};
use dap_mcp_sidecar::nvim::{BridgeShared, NvimBridge, NvimHandler};

/// Parses configuration, connects to parent Neovim, and serves MCP.
#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => std::process::exit(0),
        Err(error) => {
            eprintln!("[{}] {error}", error.code());
            std::process::exit(1);
        }
    }
}

/// Runs the composed sidecar until Neovim, HTTP, or shutdown completes.
async fn run() -> Result<(), SidecarError> {
    let config = RuntimeConfig::try_from(Cli::parse())?;
    logging::initialize(config.log_level);
    let timeout = Duration::from_secs(config.timeout_seconds);
    let shared = BridgeShared::new();
    let handler = NvimHandler::new(shared.clone());
    let (neovim, mut io_task) =
        nvim_rs::create::tokio::new_parent(handler)
            .await
            .map_err(|error| SidecarError::Runtime {
                message: format!("failed to connect to parent Neovim: {error}"),
            })?;
    let bridge = NvimBridge::new(neovim, shared.clone(), timeout).await?;
    let runtime = DebugRuntime::new(bridge, shared.clone(), timeout, config.allow_external_files);
    let shutdown = shared.shutdown.clone();
    let http = serve_http(config, runtime, shutdown.clone());
    tokio::pin!(http);
    let http_finished = tokio::select! {
        result = &mut http => {
            result?;
            true
        },
        result = &mut io_task => {
            result.map_err(|error| SidecarError::Runtime { message: error.to_string() })?
                .map_err(|error| SidecarError::Runtime { message: error.to_string() })?;
            false
        }
        () = shutdown.cancelled() => false
    };
    shutdown.cancel();
    if !http_finished {
        let _graceful_http = tokio::time::timeout(Duration::from_secs(1), &mut http).await;
    }
    io_task.abort();
    Ok(())
}
