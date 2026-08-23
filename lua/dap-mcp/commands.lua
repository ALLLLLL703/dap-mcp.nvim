---@class DapMcpCommandArgs
---@field args string

local M = {}

---@type DapMcpLifecycle?
local runtime
---@type DapMcpLogger?
local logger
local registered = false

---Reports a structured command failure.
---@param err DapMcpError?
---@return nil
local function report_error(err)
  if not logger or not err then
    return
  end
  logger:log("error", "command.failed", tostring(err))
end

---Starts the configured runtime from a user command.
---@param args DapMcpCommandArgs
---@return nil
local function start_command(args)
  local active_runtime = runtime
  if not active_runtime then
    return
  end
  local started, err = active_runtime:start(tonumber(args.args))
  if not started then
    report_error(err)
  end
end

---Stops the configured runtime from a user command.
---@return nil
local function stop_command()
  local active_runtime = runtime
  if not active_runtime then
    return
  end
  local stopping, err = active_runtime:stop()
  if not stopping then
    report_error(err)
  end
end

---Registers commands once and updates their runtime collaborators.
---@param new_runtime DapMcpLifecycle
---@param new_logger DapMcpLogger
---@return nil
function M.configure(new_runtime, new_logger)
  runtime = new_runtime
  logger = new_logger
  if registered then
    return
  end
  vim.api.nvim_create_user_command("DebuggerMcpStart", start_command, {
    desc = "Start dap-mcp.nvim on a required loopback port",
    nargs = 1,
  })
  vim.api.nvim_create_user_command("DebuggerMcpStop", stop_command, {
    desc = "Stop dap-mcp.nvim and its active debug session",
  })
  registered = true
end

return M
