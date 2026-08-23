use std::path::PathBuf;

use thiserror::Error;

/// Stable sidecar failures mapped to actionable MCP errors.
#[derive(Debug, Error)]
pub enum SidecarError {
    /// A sidecar composition, transport, or task failed.
    #[error("Sidecar runtime failed: {message}")]
    Runtime {
        /// Actionable runtime failure.
        message: String,
    },
    /// A command-line or runtime configuration value is invalid.
    #[error("{message}")]
    InvalidConfig {
        /// Actionable validation message.
        message: String,
    },
    /// A requested path is outside the declared workspace.
    #[error("File '{file}' is outside working directory '{workspace}'")]
    FileOutsideWorkspace {
        /// Canonical file path.
        file: PathBuf,
        /// Canonical workspace path.
        workspace: PathBuf,
    },
    /// A required file could not be read.
    #[error("Failed to read '{path}': {source}")]
    FileRead {
        /// File that failed to load.
        path: PathBuf,
        /// Underlying filesystem failure.
        source: std::io::Error,
    },
    /// JSONC parsing or deserialization failed.
    #[error("Invalid launch.json: {message}")]
    InvalidLaunchJson {
        /// Parser or schema failure.
        message: String,
    },
    /// No launch configuration matched the requested name.
    #[error("Configuration '{name}' was not found; available configurations: {available:?}")]
    LaunchConfigurationMissing {
        /// Requested configuration name.
        name: String,
        /// Available configuration names.
        available: Vec<String>,
    },
    /// More than one launch configuration used the requested name.
    #[error("Configuration name '{name}' is duplicated")]
    LaunchConfigurationDuplicate {
        /// Duplicated configuration name.
        name: String,
    },
    /// A launch configuration has an unsupported request kind.
    #[error("Configuration '{name}' must use request 'launch' or 'attach'")]
    UnsupportedLaunchRequest {
        /// Invalid configuration name.
        name: String,
    },
    /// Neovim has no cross-language equivalent of the VS Code Testing API.
    #[error("testName is not supported; select a launch.json configuration instead")]
    UnsupportedTestTarget,
    /// A debug session transition violates the single-session state machine.
    #[error("Invalid debug state transition from {from} to {to}")]
    InvalidDebugTransition {
        /// Previous state.
        from: &'static str,
        /// Requested next state.
        to: &'static str,
    },
    /// A value could not cross the Msgpack-RPC boundary.
    #[error("RPC value conversion failed: {message}")]
    RpcValue {
        /// Conversion failure.
        message: String,
    },
    /// Neovim rejected or disconnected an RPC call.
    #[error("Neovim RPC call failed: {message}")]
    RpcCall {
        /// Underlying RPC failure.
        message: String,
    },
    /// Neovim did not answer within the configured timeout.
    #[error("Neovim RPC request '{method}' timed out")]
    RpcTimeout {
        /// Timed-out bridge method.
        method: String,
    },
    /// Lua returned a structured operation error.
    #[error("{message}")]
    Remote {
        /// Stable Lua-side error code.
        code: String,
        /// Actionable Lua-side error message.
        message: String,
    },
}

impl SidecarError {
    /// Returns the stable machine-readable error code.
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Runtime { .. } => "SIDECAR_RUNTIME_FAILED",
            Self::InvalidConfig { .. } => "INVALID_CONFIG",
            Self::FileOutsideWorkspace { .. } => "FILE_OUTSIDE_WORKSPACE",
            Self::FileRead { .. } => "FILE_READ_FAILED",
            Self::InvalidLaunchJson { .. } => "INVALID_LAUNCH_JSON",
            Self::LaunchConfigurationMissing { .. } => "LAUNCH_CONFIGURATION_MISSING",
            Self::LaunchConfigurationDuplicate { .. } => "LAUNCH_CONFIGURATION_DUPLICATE",
            Self::UnsupportedLaunchRequest { .. } => "UNSUPPORTED_LAUNCH_REQUEST",
            Self::UnsupportedTestTarget => "UNSUPPORTED_TEST_TARGET",
            Self::InvalidDebugTransition { .. } => "INVALID_DEBUG_TRANSITION",
            Self::RpcValue { .. } => "RPC_VALUE_ERROR",
            Self::RpcCall { .. } => "RPC_CALL_FAILED",
            Self::RpcTimeout { .. } => "RPC_TIMEOUT",
            Self::Remote { code, .. } => code,
        }
    }
}
