use rmcp::{
    ErrorData, Json, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use serde::Serialize;
use serde_json::{Value, json};

use super::DebugRuntime;
use super::schema::{
    AddBreakpointArgs, AddLogpointArgs, DapVariable, EvaluateExpressionArgs,
    GetVariablesValuesArgs, ListVariableNamesArgs, LuaBreakpointArgs, LuaRemoveBreakpointArgs,
    RemoveBreakpointArgs, StartDebuggingArgs, VariableScope,
};
use crate::error::SidecarError;
use crate::nvim::DebugEvent;

/// Per-MCP-session debugger tool handler sharing one Neovim runtime.
#[derive(Clone)]
pub struct DebugMcpServer {
    runtime: Option<DebugRuntime>,
    tool_router: ToolRouter<Self>,
}

impl DebugMcpServer {
    /// Creates a production handler over the active Neovim bridge.
    #[must_use]
    pub fn new(runtime: DebugRuntime) -> Self {
        Self {
            runtime: Some(runtime),
            tool_router: Self::tool_router(),
        }
    }

    /// Returns the production runtime or an internal configuration error.
    fn runtime(&self) -> Result<&DebugRuntime, ErrorData> {
        self.runtime
            .as_ref()
            .ok_or_else(|| ErrorData::internal_error("Debugger runtime is unavailable", None))
    }

    /// Runs one operation under the global backstop and maps stable errors.
    async fn run<F, T>(&self, method: &str, operation: F) -> Result<Json<T>, ErrorData>
    where
        F: std::future::Future<Output = Result<T, SidecarError>>,
        T: Serialize,
    {
        let runtime = self.runtime()?;
        runtime
            .with_backstop(method, operation)
            .await
            .map(Json)
            .map_err(mcp_error)
    }

    /// Loads and scope-filters active-frame variables.
    async fn scoped_variables(
        &self,
        scope: Option<VariableScope>,
    ) -> Result<Vec<DapVariable>, SidecarError> {
        let variables = self
            .runtime()
            .map_err(error_data_to_sidecar)?
            .variables()
            .await?;
        Ok(variables
            .into_iter()
            .filter(|variable| scope_matches(variable, scope))
            .collect())
    }

    /// Expands descendant names and types up to the project safety limit.
    async fn descendants(&self, root: &DapVariable) -> Result<Vec<Value>, SidecarError> {
        let runtime = self.runtime().map_err(error_data_to_sidecar)?;
        let mut pending = vec![(root.name.clone(), root.variables_reference)];
        let mut result = Vec::new();
        while let Some((parent, reference)) = pending.pop() {
            if reference <= 0 || result.len() >= 100 {
                continue;
            }
            for child in runtime.variable_children(reference).await? {
                if result.len() >= 100 {
                    break;
                }
                let path = format!("{parent}.{}", child.name);
                result.push(json!({ "name": path, "type": child.type_name }));
                if child.variables_reference > 0 {
                    pending.push((path, child.variables_reference));
                }
            }
        }
        Ok(result)
    }
}

#[tool_router(router = tool_router)]
impl DebugMcpServer {
    /// Starts the exact named launch.json configuration.
    #[tool(description = "Start a Neovim nvim-dap session from a named launch.json configuration.")]
    async fn start_debugging(
        &self,
        Parameters(args): Parameters<StartDebuggingArgs>,
    ) -> Result<Json<Value>, ErrorData> {
        let runtime = self.runtime()?;
        self.run("start_debugging", runtime.start(args)).await
    }

    /// Stops the active debug session.
    #[tool(description = "Stop the current debug session.")]
    async fn stop_debugging(&self) -> Result<Json<Value>, ErrorData> {
        self.bridge_value("stop_debugging", &json!({})).await
    }

    /// Steps over the current source line.
    #[tool(description = "Execute the current line without entering called functions.")]
    async fn step_over(&self) -> Result<Json<Value>, ErrorData> {
        self.control("step_over", stopped).await
    }

    /// Steps into the current call.
    #[tool(description = "Enter the function called by the current source line.")]
    async fn step_into(&self) -> Result<Json<Value>, ErrorData> {
        self.control("step_into", stopped).await
    }

    /// Steps out of the current function.
    #[tool(description = "Run until the current function returns.")]
    async fn step_out(&self) -> Result<Json<Value>, ErrorData> {
        self.control("step_out", stopped).await
    }

    /// Continues until another stop or termination.
    #[tool(description = "Resume execution until a breakpoint or program termination.")]
    async fn continue_execution(&self) -> Result<Json<Value>, ErrorData> {
        self.control("continue_execution", continued).await
    }

    /// Interrupts a freely running debuggee.
    #[tool(description = "Pause the running program at its current location.")]
    async fn pause_execution(&self) -> Result<Json<Value>, ErrorData> {
        self.control("pause_execution", stopped).await
    }

    /// Restarts with the active configuration.
    #[tool(description = "Restart the active debug session with the same configuration.")]
    async fn restart_debugging(&self) -> Result<Json<Value>, ErrorData> {
        self.control("restart_debugging", restarted).await
    }

    /// Adds a stopping source breakpoint.
    #[tool(description = "Set a one-based source breakpoint with an optional condition.")]
    async fn add_breakpoint(
        &self,
        Parameters(args): Parameters<AddBreakpointArgs>,
    ) -> Result<Json<Value>, ErrorData> {
        validate_line(args.line)?;
        let lua = LuaBreakpointArgs {
            file_path: args.file_full_path,
            line: args.line,
            condition: args.condition,
            log_message: None,
        };
        self.bridge_value("add_breakpoint", &lua).await
    }

    /// Adds a non-stopping logpoint.
    #[tool(description = "Set a logpoint that emits an interpolated message without stopping.")]
    async fn add_logpoint(
        &self,
        Parameters(args): Parameters<AddLogpointArgs>,
    ) -> Result<Json<Value>, ErrorData> {
        validate_line(args.line)?;
        let lua = LuaBreakpointArgs {
            file_path: args.file_full_path,
            line: args.line,
            condition: args.condition,
            log_message: Some(args.log_message),
        };
        self.bridge_value("add_breakpoint", &lua).await
    }

    /// Removes one exact source breakpoint.
    #[tool(description = "Remove the breakpoint at an exact file and one-based line.")]
    async fn remove_breakpoint(
        &self,
        Parameters(args): Parameters<RemoveBreakpointArgs>,
    ) -> Result<Json<Value>, ErrorData> {
        validate_line(args.line)?;
        let lua = LuaRemoveBreakpointArgs {
            file_path: args.file_full_path,
            line: args.line,
        };
        self.bridge_value("remove_breakpoint", &lua).await
    }

    /// Clears every breakpoint.
    #[tool(description = "Clear every nvim-dap breakpoint.")]
    async fn clear_all_breakpoints(&self) -> Result<Json<Value>, ErrorData> {
        self.bridge_value("clear_all_breakpoints", &json!({})).await
    }

    /// Lists every configured breakpoint.
    #[tool(description = "List all current breakpoints across source files.")]
    async fn list_breakpoints(&self) -> Result<Json<Value>, ErrorData> {
        self.bridge_value("list_breakpoints", &json!({})).await
    }

    /// Lists visible variable names and types without values.
    #[tool(
        description = "List names and types visible at the stopped frame without returning values."
    )]
    async fn list_variable_names(
        &self,
        Parameters(args): Parameters<ListVariableNamesArgs>,
    ) -> Result<Json<Value>, ErrorData> {
        let values = self.scoped_variables(args.scope).await.map_err(mcp_error)?;
        Ok(Json(json!(
            values
                .into_iter()
                .map(
                    |item| json!({ "name": item.name, "type": item.type_name, "scope": item.scope })
                )
                .collect::<Vec<_>>()
        )))
    }

    /// Retrieves exact requested variable values and bounded descendants.
    #[tool(
        description = "Read one to fifty exact variable names and up to 100 descendant names and types."
    )]
    async fn get_variables_values(
        &self,
        Parameters(args): Parameters<GetVariablesValuesArgs>,
    ) -> Result<Json<Value>, ErrorData> {
        if args.variable_names.is_empty() || args.variable_names.len() > 50 {
            return Err(ErrorData::invalid_params(
                "variableNames must contain 1 to 50 exact names",
                None,
            ));
        }
        let variables = self.scoped_variables(args.scope).await.map_err(mcp_error)?;
        let mut selected = Vec::new();
        for name in args.variable_names {
            if let Some(variable) = variables.iter().find(|item| item.name == name) {
                selected.push(json!({ "name": variable.name, "value": variable.value, "type": variable.type_name, "descendants": self.descendants(variable).await.map_err(mcp_error)? }));
            } else {
                selected.push(json!({ "name": name, "missing": true }));
            }
        }
        Ok(Json(json!(selected)))
    }

    /// Evaluates a target-language expression in the current frame.
    #[tool(description = "Evaluate any valid target-language expression in the stopped frame.")]
    async fn evaluate_expression(
        &self,
        Parameters(args): Parameters<EvaluateExpressionArgs>,
    ) -> Result<Json<Value>, ErrorData> {
        self.bridge_value("evaluate", &json!({ "expression": args.expression }))
            .await
    }
}

impl DebugMcpServer {
    /// Runs one control action and awaits its DAP event.
    async fn control(
        &self,
        method: &str,
        expected: fn(&DebugEvent) -> bool,
    ) -> Result<Json<Value>, ErrorData> {
        let runtime = self.runtime()?;
        self.run(method, runtime.action(method, expected)).await
    }

    /// Runs one immediate typed Lua bridge operation.
    async fn bridge_value<T: Serialize>(
        &self,
        method: &str,
        args: &T,
    ) -> Result<Json<Value>, ErrorData> {
        let runtime = self.runtime()?;
        self.run(method, runtime.call(method, args)).await
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for DebugMcpServer {
    /// Advertises the tool-only server and local debugger safety guidance.
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("dap-mcp.nvim", env!("CARGO_PKG_VERSION")))
            .with_instructions("These tools control the current Neovim nvim-dap client. The endpoint is trusted-local only; evaluate_expression can execute target-language code.")
    }
}

/// Maps sidecar errors to MCP errors while preserving stable codes as data.
fn mcp_error(error: SidecarError) -> ErrorData {
    ErrorData::internal_error(error.to_string(), Some(json!({ "code": error.code() })))
}

/// Rejects zero before source positions cross into Lua.
fn validate_line(line: u32) -> Result<(), ErrorData> {
    if line == 0 {
        return Err(ErrorData::invalid_params(
            "line must be a one-based positive integer",
            None,
        ));
    }
    Ok(())
}

/// Converts an already structured MCP setup error for internal helpers.
fn error_data_to_sidecar(error: ErrorData) -> SidecarError {
    SidecarError::InvalidConfig {
        message: error.to_string(),
    }
}

/// Returns whether a variable belongs to the requested adapter scope class.
fn scope_matches(variable: &DapVariable, requested: Option<VariableScope>) -> bool {
    let global = variable.scope.as_deref().is_some_and(|name| {
        let name = name.to_ascii_lowercase();
        name.contains("global") || name.contains("static")
    });
    match requested.unwrap_or(VariableScope::All) {
        VariableScope::All => true,
        VariableScope::Local => !global,
        VariableScope::Global => global,
    }
}

/// Matches a stopped DAP event.
fn stopped(event: &DebugEvent) -> bool {
    matches!(event, DebugEvent::Stopped { .. })
}

/// Matches execution progress or termination.
fn continued(event: &DebugEvent) -> bool {
    matches!(event, DebugEvent::Continued | DebugEvent::Terminated)
}

/// Matches a restarted session reaching a meaningful lifecycle state.
fn restarted(event: &DebugEvent) -> bool {
    matches!(
        event,
        DebugEvent::SessionStarted { .. } | DebugEvent::Stopped { .. } | DebugEvent::Continued
    )
}

#[cfg(test)]
mod tests {
    use rmcp::handler::server::router::tool::ToolRouter;

    use super::DebugMcpServer;

    /// Exposes all sixteen compatibility tool names.
    #[test]
    fn exposes_sixteen_tools() {
        let server = DebugMcpServer {
            runtime: None,
            tool_router: ToolRouter::new(),
        };
        let names = DebugMcpServer::tool_router().list_all();
        assert_eq!(names.len(), 16);
        assert!(names.iter().any(|tool| tool.name == "start_debugging"));
        let breakpoint = names
            .iter()
            .find(|tool| tool.name == "add_breakpoint")
            .expect("breakpoint tool");
        let breakpoint_schema = serde_json::to_string(&breakpoint.input_schema).expect("schema");
        assert!(breakpoint_schema.contains("\"minimum\":1"));
        let variables = names
            .iter()
            .find(|tool| tool.name == "get_variables_values")
            .expect("variables tool");
        let variables_schema = serde_json::to_string(&variables.input_schema).expect("schema");
        assert!(variables_schema.contains("\"minItems\":1"));
        assert!(variables_schema.contains("\"maxItems\":50"));
        drop(server);
    }
}
