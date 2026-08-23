local errors = require("dap-mcp.errors")
local launch = require("dap-mcp.launch")
local requests = require("dap-mcp.dap_requests")

local M = {}

---Loads nvim-dap only when debugger functionality is requested.
---@return boolean available
---@return table? dap
local function load_dap()
  local available, dap = pcall(require, "dap")
  return available, available and dap or nil
end

---Returns nvim-dap or a stable dependency error.
---@return table? dap
---@return DapMcpError? error
local function required_dap()
  local available, dap = load_dap()
  if not available or not dap then
    return nil, errors.new("NVIM_DAP_MISSING", "nvim-dap is required to use debugger tools")
  end
  return dap, nil
end

---Returns the active DAP session or a stable state error.
---@param dap table
---@return table? session
---@return DapMcpError? error
local function active_session(dap)
  local session = dap.session()
  if not session then
    return nil, errors.new("NO_ACTIVE_SESSION", "No debug session is active")
  end
  return session, nil
end

---Checks whether nvim-dap is installed.
---@return boolean
function M.is_available()
  return load_dap()
end

---Starts a selected configuration.
---@param arguments table<string, unknown>
---@return table<string, unknown>? result
---@return DapMcpError? error
function M.start(arguments)
  local dap, err = required_dap()
  if err then
    return nil, err
  end
  assert(dap, "required_dap must return dap without an error")
  return launch.start(dap, arguments.configuration, arguments.auto_open_dap_ui == true)
end

---Runs a stateful nvim-dap action.
---@param action string
---@return table<string, boolean>? result
---@return DapMcpError? error
function M.action(action)
  local dap, err = required_dap()
  if err then
    return nil, err
  end
  assert(dap, "required_dap must return dap without an error")
  local _, session_error = active_session(dap)
  if session_error then
    return nil, session_error
  end
  local actions = {
    continue_execution = dap.continue,
    restart_debugging = dap.restart,
    step_over = dap.step_over,
    step_into = dap.step_into,
    step_out = dap.step_out,
  }
  local operation = actions[action]
  if not operation then
    return nil, errors.new("UNKNOWN_DAP_ACTION", "Unknown debugger action: " .. action)
  end
  local ok, failure = pcall(operation)
  if not ok then
    return nil, errors.new("DAP_ACTION_FAILED", tostring(failure), { action = action })
  end
  return { accepted = true }, nil
end

---Stops the active session after nvim-dap finishes terminate or disconnect.
---@param callback fun(result: table<string, boolean>?, error: DapMcpError?)
---@return nil
function M.stop(callback)
  local dap, err = required_dap()
  if err then
    callback(nil, err)
    return
  end
  assert(dap, "required_dap must return dap without an error")
  local _, session_error = active_session(dap)
  if session_error then
    callback(nil, session_error)
    return
  end
  local ok, failure = pcall(dap.terminate, {
    on_done = function()
      callback({ accepted = true }, nil)
    end,
  })
  if not ok then
    callback(nil, errors.new("DAP_ACTION_FAILED", tostring(failure), { action = "stop_debugging" }))
  end
end

---Pauses the first adapter-reported thread without opening a selector UI.
---@param callback fun(result: table<string, boolean>?, error: DapMcpError?)
---@return nil
function M.pause(callback)
  local dap, err = required_dap()
  if err then
    callback(nil, err)
    return
  end
  assert(dap, "required_dap must return dap without an error")
  local session, session_error = active_session(dap)
  if session_error then
    callback(nil, session_error)
    return
  end
  assert(session, "active_session must return a session without an error")
  session:request("threads", nil, function(failure, response)
    if failure then
      callback(
        nil,
        errors.new("DAP_REQUEST_FAILED", "threads failed: " .. tostring(failure.message or failure))
      )
      return
    end
    local thread = (response and response.threads or {})[1]
    if not thread then
      callback(nil, errors.new("NO_DEBUG_THREAD", "The debug adapter reported no thread to pause"))
      return
    end
    local paused, pause_failure = pcall(dap.pause, thread.id)
    if not paused then
      callback(
        nil,
        errors.new("DAP_ACTION_FAILED", tostring(pause_failure), { action = "pause_execution" })
      )
      return
    end
    callback({ accepted = true }, nil)
  end)
end

---Terminates the active nvim-dap session when one exists.
---@return nil
function M.terminate()
  local available, dap = load_dap()
  if available and dap and dap.session() then
    dap.terminate()
  end
end

---Resolves or creates a loaded buffer for an absolute source path.
---@param file_path string
---@return integer bufnr
local function source_buffer(file_path)
  local bufnr = vim.fn.bufnr(file_path)
  if bufnr < 0 then
    bufnr = vim.fn.bufadd(file_path)
  end
  vim.fn.bufload(bufnr)
  return bufnr
end

---Synchronizes one buffer's breakpoints to the active session when present.
---@param dap table
---@param breakpoints table
---@param bufnr integer
---@return nil
local function sync_breakpoints(dap, breakpoints, bufnr)
  local session = dap.session()
  if session then
    session:set_breakpoints(breakpoints.get(bufnr))
  end
end

---Adds or replaces one breakpoint or logpoint.
---@param arguments table<string, unknown>
---@return table<string, unknown>? result
---@return DapMcpError? error
function M.add_breakpoint(arguments)
  local dap, err = required_dap()
  if err then
    return nil, err
  end
  assert(dap, "required_dap must return dap without an error")
  if type(arguments.file_path) ~= "string" or type(arguments.line) ~= "number" then
    return nil, errors.new("INVALID_BREAKPOINT", "file_path and line are required")
  end
  local breakpoints = require("dap.breakpoints")
  local bufnr = source_buffer(arguments.file_path)
  breakpoints.set({
    condition = arguments.condition,
    hit_condition = arguments.hit_condition,
    log_message = arguments.log_message,
  }, bufnr, arguments.line)
  sync_breakpoints(dap, breakpoints, bufnr)
  return { fileFullPath = arguments.file_path, line = arguments.line }, nil
end

---Removes one breakpoint at an exact file and line.
---@param arguments table<string, unknown>
---@return table<string, boolean>? result
---@return DapMcpError? error
function M.remove_breakpoint(arguments)
  local dap, err = required_dap()
  if err then
    return nil, err
  end
  assert(dap, "required_dap must return dap without an error")
  local breakpoints = require("dap.breakpoints")
  local bufnr = source_buffer(arguments.file_path)
  breakpoints.remove(bufnr, arguments.line)
  sync_breakpoints(dap, breakpoints, bufnr)
  return { removed = true }, nil
end

---Clears all breakpoints and synchronizes the active session.
---@return table<string, boolean>? result
---@return DapMcpError? error
function M.clear_breakpoints()
  local dap, err = required_dap()
  if err then
    return nil, err
  end
  assert(dap, "required_dap must return dap without an error")
  dap.clear_breakpoints()
  return { cleared = true }, nil
end

---Lists all breakpoints using absolute source paths.
---@return table[]? result
---@return DapMcpError? error
function M.list_breakpoints()
  local _, err = required_dap()
  if err then
    return nil, err
  end
  local result = {}
  for bufnr, entries in pairs(require("dap.breakpoints").get()) do
    for _, entry in ipairs(entries) do
      table.insert(result, {
        fileFullPath = vim.api.nvim_buf_get_name(bufnr),
        line = entry.line,
        condition = entry.condition,
        hitCondition = entry.hitCondition,
        logMessage = entry.logMessage,
      })
    end
  end
  return result, nil
end

---Loads active-frame variables asynchronously.
---@param callback fun(result: table[]?, error: DapMcpError?)
---@return nil
function M.variables(callback)
  local dap, err = required_dap()
  if err then
    callback(nil, err)
    return
  end
  assert(dap, "required_dap must return dap without an error")
  requests.variables(dap, callback)
end

---Loads direct children of one structured variable asynchronously.
---@param variables_reference integer
---@param callback fun(result: table[]?, error: DapMcpError?)
---@return nil
function M.variable_children(variables_reference, callback)
  local dap, err = required_dap()
  if err then
    callback(nil, err)
    return
  end
  assert(dap, "required_dap must return dap without an error")
  requests.children(dap, variables_reference, callback)
end

---Evaluates an expression asynchronously in the active frame.
---@param expression string
---@param callback fun(result: table<string, unknown>?, error: DapMcpError?)
---@return nil
function M.evaluate(expression, callback)
  local dap, err = required_dap()
  if err then
    callback(nil, err)
    return
  end
  assert(dap, "required_dap must return dap without an error")
  requests.evaluate(dap, expression, callback)
end

return M
