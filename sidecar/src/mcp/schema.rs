use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Arguments for selecting and starting one launch.json configuration.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StartDebuggingArgs {
    /// Full source path used for workspace containment validation.
    pub file_full_path: PathBuf,
    /// Workspace containing .vscode/launch.json.
    pub working_directory: PathBuf,
    /// Exact configuration name from launch.json.
    pub configuration_name: String,
    /// Upstream-compatible test target, unsupported in the first release.
    pub test_name: Option<String>,
}

/// Arguments for an ordinary source breakpoint.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddBreakpointArgs {
    /// Absolute source file path.
    pub file_full_path: String,
    /// One-based source line.
    #[schemars(range(min = 1))]
    pub line: u32,
    /// Optional conditional expression.
    pub condition: Option<String>,
}

/// Arguments for a non-stopping logpoint.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddLogpointArgs {
    /// Absolute source file path.
    pub file_full_path: String,
    /// One-based source line.
    #[schemars(range(min = 1))]
    pub line: u32,
    /// Adapter-specific interpolation message.
    pub log_message: String,
    /// Optional conditional expression.
    pub condition: Option<String>,
}

/// Arguments identifying one exact breakpoint.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoveBreakpointArgs {
    /// Absolute source file path.
    pub file_full_path: String,
    /// One-based source line.
    #[schemars(range(min = 1))]
    pub line: u32,
}

/// Supported variable scope filters.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum VariableScope {
    /// Non-global adapter scopes.
    Local,
    /// Adapter scopes named global or static.
    Global,
    /// Every non-expensive scope.
    All,
}

/// Arguments for variable-name discovery.
#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ListVariableNamesArgs {
    /// Optional scope filter.
    pub scope: Option<VariableScope>,
}

/// Arguments for exact variable value retrieval.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetVariablesValuesArgs {
    /// One to fifty exact variable names.
    #[schemars(length(min = 1, max = 50))]
    pub variable_names: Vec<String>,
    /// Optional scope filter.
    pub scope: Option<VariableScope>,
}

/// Arguments for runtime expression evaluation.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct EvaluateExpressionArgs {
    /// Target-language expression.
    pub expression: String,
}

/// Internal Lua launch request.
#[derive(Debug, Serialize)]
pub struct LuaStartArgs {
    /// Selected launch configuration with adapter fields preserved.
    pub configuration: Value,
}

/// Internal Lua breakpoint request.
#[derive(Debug, Serialize)]
pub struct LuaBreakpointArgs {
    /// Absolute source path.
    pub file_path: String,
    /// One-based source line.
    pub line: u32,
    /// Optional condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Optional logpoint message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_message: Option<String>,
}

/// Internal Lua breakpoint removal request.
#[derive(Debug, Serialize)]
pub struct LuaRemoveBreakpointArgs {
    /// Absolute source path.
    pub file_path: String,
    /// One-based source line.
    pub line: u32,
}

/// A DAP variable returned by nvim-dap.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DapVariable {
    /// Variable display name.
    pub name: String,
    /// Adapter-rendered value.
    pub value: String,
    /// Optional adapter type name.
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    /// Child reference, zero for scalar values.
    #[serde(default)]
    pub variables_reference: i64,
    /// nvim-dap scope name annotated by the Lua bridge.
    pub scope: Option<String>,
}

/// Internal request for variable children.
#[derive(Debug, Serialize)]
pub struct VariableChildrenArgs {
    /// DAP variablesReference value.
    pub variables_reference: i64,
}
