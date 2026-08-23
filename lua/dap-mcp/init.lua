local commands = require("dap-mcp.commands")
local config = require("dap-mcp.config")
local dap_client = require("dap-mcp.dap_client")
local errors = require("dap-mcp.errors")
local lifecycle = require("dap-mcp.lifecycle")
local logger = require("dap-mcp.logger")
local process = require("dap-mcp.process")
local ui = require("dap-mcp.ui")

local M = {}

---@type DapMcpLifecycle?
local runtime

---@type DapMcpConfig?
local active_config

---Builds the sidecar lifecycle dependencies at the application boundary.
---@return DapMcpLifecycleDeps
local function lifecycle_dependencies()
  return {
    is_dap_available = dap_client.is_available,
    is_executable = process.is_executable,
    spawn = process.spawn,
    stop_debugging = dap_client.terminate,
    close_ui = ui.close,
    request_shutdown = process.request_shutdown,
  }
end

---Configures dap-mcp.nvim without starting its MCP server.
---
---Example:
---```lua
---require("dap-mcp").setup({ binary_path = "/path/to/dap-mcp-sidecar" })
---```
---@param options? table<string, unknown>
---@return boolean configured
---@return DapMcpError? error
function M.setup(options)
  if runtime and (runtime:state() == "running" or runtime:state() == "stopping") then
    return false,
      errors.new("CONFIG_WHILE_RUNNING", "Stop dap-mcp.nvim before changing its configuration")
  end
  local resolved, err = config.resolve(options)
  if not resolved then
    return false, err
  end
  local configured_logger = logger.new(resolved.log_level)
  active_config = vim.deepcopy(resolved)
  runtime = lifecycle.new(resolved, lifecycle_dependencies(), configured_logger)
  commands.configure(runtime, configured_logger)
  return true, nil
end

---Returns an immutable copy of the validated plugin configuration.
---@return DapMcpConfig? config
function M.config_snapshot()
  return active_config and vim.deepcopy(active_config) or nil
end

---Returns whether setup has produced a validated runtime snapshot.
---@return boolean
function M.is_configured()
  return active_config ~= nil
end

---Returns the runtime state for diagnostics and tests.
---@return DapMcpLifecycleState
function M.state()
  return runtime and runtime:state() or "stopped"
end

return M
