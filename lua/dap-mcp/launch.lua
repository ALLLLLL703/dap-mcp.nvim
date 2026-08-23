local errors = require("dap-mcp.errors")
local ui = require("dap-mcp.ui")

local M = {}

---Starts one exact launch.json configuration through nvim-dap.
---@param dap table
---@param configuration table<string, unknown>
---@param auto_open_ui boolean
---@return table<string, unknown>? result
---@return DapMcpError? error
function M.start(dap, configuration, auto_open_ui)
  if dap.session() then
    return nil, errors.new("DEBUG_SESSION_ACTIVE", "A debug session is already active")
  end
  if type(configuration) ~= "table" or type(configuration.name) ~= "string" then
    return nil,
      errors.new("INVALID_LAUNCH_CONFIGURATION", "A named launch configuration is required")
  end
  local copied = vim.deepcopy(configuration)
  local ok, failure = pcall(dap.run, copied)
  if not ok then
    return nil, errors.new("DEBUG_START_FAILED", tostring(failure))
  end
  ui.open(auto_open_ui)
  return { accepted = true, configurationName = copied.name }, nil
end

return M
