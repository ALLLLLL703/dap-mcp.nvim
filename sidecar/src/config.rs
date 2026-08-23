use clap::{Parser, ValueEnum};

use crate::error::SidecarError;

/// Supported structured log levels.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum LogLevel {
    /// Diagnostic protocol details.
    Debug,
    /// Normal lifecycle events.
    Info,
    /// Recoverable problems requiring attention.
    Warn,
    /// Failed operations.
    Error,
}

/// Raw command-line arguments sent by the Neovim plugin.
#[derive(Clone, Debug, Parser)]
#[command(name = "dap-mcp-sidecar")]
pub struct Cli {
    /// Required MCP listening port.
    #[arg(long)]
    pub port: u16,
    /// Loopback listening host.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// Per-operation timeout in seconds.
    #[arg(long, default_value_t = 180)]
    pub timeout_seconds: u64,
    /// Minimum structured log level.
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,
    /// Whether source paths outside the workspace are accepted.
    #[arg(long, default_value_t = false)]
    pub allow_external_files: bool,
}

/// Validated immutable sidecar configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeConfig {
    /// Non-zero TCP port.
    pub port: u16,
    /// Validated loopback host.
    pub host: String,
    /// Positive operation timeout.
    pub timeout_seconds: u64,
    /// Structured log level.
    pub log_level: LogLevel,
    /// Validated workspace containment policy.
    pub allow_external_files: bool,
}

impl TryFrom<Cli> for RuntimeConfig {
    type Error = SidecarError;

    /// Validates raw CLI values into a runtime snapshot.
    fn try_from(cli: Cli) -> Result<Self, Self::Error> {
        if cli.port == 0 {
            return Err(SidecarError::InvalidConfig {
                message: "port must be between 1 and 65535".to_owned(),
            });
        }
        if !matches!(cli.host.as_str(), "localhost" | "127.0.0.1" | "::1") {
            return Err(SidecarError::InvalidConfig {
                message: "host must be localhost, 127.0.0.1, or ::1".to_owned(),
            });
        }
        if cli.timeout_seconds == 0 {
            return Err(SidecarError::InvalidConfig {
                message: "timeout-seconds must be positive".to_owned(),
            });
        }
        Ok(Self {
            port: cli.port,
            host: cli.host,
            timeout_seconds: cli.timeout_seconds,
            log_level: cli.log_level,
            allow_external_files: cli.allow_external_files,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, LogLevel, RuntimeConfig};

    /// Creates valid raw arguments for focused mutation.
    fn valid_cli() -> Cli {
        Cli {
            port: 3001,
            host: "127.0.0.1".to_owned(),
            timeout_seconds: 180,
            log_level: LogLevel::Info,
            allow_external_files: false,
        }
    }

    /// Accepts a valid loopback configuration.
    #[test]
    fn accepts_valid_config() {
        let result = RuntimeConfig::try_from(valid_cli());
        assert!(result.is_ok());
    }

    /// Rejects port zero before binding.
    #[test]
    fn rejects_zero_port() {
        let mut cli = valid_cli();
        cli.port = 0;
        let error = RuntimeConfig::try_from(cli).expect_err("port zero must fail");
        assert_eq!(error.code(), "INVALID_CONFIG");
    }

    /// Rejects a non-loopback listening host.
    #[test]
    fn rejects_non_loopback_host() {
        let mut cli = valid_cli();
        cli.host = "0.0.0.0".to_owned();
        assert!(RuntimeConfig::try_from(cli).is_err());
    }
}
