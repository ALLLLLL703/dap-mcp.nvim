local errors = require("dap-mcp.errors")

local M = {}

---Returns the active session and stopped frame or a structured error.
---@param dap table
---@return table? session
---@return table? frame
---@return DapMcpError? error
local function active_frame(dap)
  local session = dap.session()
  if not session then
    return nil, nil, errors.new("NO_ACTIVE_SESSION", "No debug session is active")
  end
  if not session.current_frame then
    return nil, nil, errors.new("NO_ACTIVE_FRAME", "The debuggee must be stopped at a stack frame")
  end
  return session, session.current_frame, nil
end

---Converts a DAP callback failure into the stable plugin error shape.
---@param operation string
---@param failure unknown
---@return DapMcpError
local function request_error(operation, failure)
  local message = type(failure) == "table" and failure.message or tostring(failure)
  return errors.new("DAP_REQUEST_FAILED", operation .. " failed: " .. message)
end

---Evaluates one expression in the active frame.
---@param dap table
---@param expression string
---@param callback fun(result: table<string, unknown>?, error: DapMcpError?)
---@return nil
function M.evaluate(dap, expression, callback)
  local session, frame, err = active_frame(dap)
  if err then
    callback(nil, err)
    return
  end
  assert(session and frame, "active_frame must return a session and frame without an error")
  session:request("evaluate", {
    expression = expression,
    frameId = frame.id,
    context = "watch",
  }, function(failure, response)
    if failure then
      callback(nil, request_error("evaluate", failure))
      return
    end
    callback(response or {}, nil)
  end)
end

---Loads all variables from the active frame's non-expensive scopes.
---@param dap table
---@param callback fun(result: table[]?, error: DapMcpError?)
---@return nil
function M.variables(dap, callback)
  local completed = false

  ---Completes the asynchronous request exactly once.
  ---@param result table[]?
  ---@param failure DapMcpError?
  local function finish(result, failure)
    if completed then
      return
    end
    completed = true
    callback(result, failure)
  end

  local session, frame, err = active_frame(dap)
  if err then
    finish(nil, err)
    return
  end
  assert(session and frame, "active_frame must return a session and frame without an error")
  session:request("scopes", { frameId = frame.id }, function(scope_failure, scope_response)
    if scope_failure then
      finish(nil, request_error("scopes", scope_failure))
      return
    end
    local scopes = vim.tbl_filter(function(scope)
      return not scope.expensive
    end, (scope_response or {}).scopes or {})
    local remaining = #scopes
    local variables = {}
    if remaining == 0 then
      finish(variables, nil)
      return
    end
    for _, scope in ipairs(scopes) do
      session:request(
        "variables",
        { variablesReference = scope.variablesReference },
        function(failure, response)
          if failure then
            finish(nil, request_error("variables", failure))
            return
          end
          for _, variable in ipairs((response or {}).variables or {}) do
            variable.scope = scope.name
          end
          vim.list_extend(variables, (response or {}).variables or {})
          remaining = remaining - 1
          if remaining == 0 then
            finish(variables, nil)
          end
        end
      )
    end
  end)
end

---Loads direct children for one DAP variables reference.
---@param dap table
---@param variables_reference integer
---@param callback fun(result: table[]?, error: DapMcpError?)
---@return nil
function M.children(dap, variables_reference, callback)
  local session = dap.session()
  if not session then
    callback(nil, errors.new("NO_ACTIVE_SESSION", "No debug session is active"))
    return
  end
  session:request(
    "variables",
    { variablesReference = variables_reference },
    function(failure, response)
      if failure then
        callback(nil, request_error("variables", failure))
        return
      end
      callback((response or {}).variables or {}, nil)
    end
  )
end

return M
