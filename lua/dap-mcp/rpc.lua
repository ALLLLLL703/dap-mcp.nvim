local dap_client = require("dap-mcp.dap_client")
local dap_events = require("dap-mcp.dap_events")
local errors = require("dap-mcp.errors")

local M = {}

---@type integer?
local response_channel

---Notifies the sidecar with a successful or failed response envelope.
---@param request_id integer
---@param result unknown
---@param err DapMcpError?
---@return nil
local function respond(request_id, result, err)
  local envelope
  if err then
    envelope = { ok = false, error = { code = err.code, message = err.message } }
  else
    envelope = { ok = true, result = result or vim.empty_dict() }
  end
  vim.rpcnotify(response_channel, "dap_mcp_response", request_id, envelope)
end

---Registers a sidecar channel and installs DAP event forwarding.
---@param channel integer
---@return boolean registered
function M.register(channel)
  assert(type(channel) == "number", "sidecar channel must be numeric")
  response_channel = channel
  dap_events.register(channel)
  return true
end

---Dispatches one bridge method without blocking Neovim's main loop.
---@param method string
---@param request_id integer
---@param arguments? table<string, unknown>
---@return boolean accepted
function M.dispatch(method, request_id, arguments)
  if not response_channel then
    return false
  end
  arguments = arguments or {}
  if method == "variables" then
    dap_client.variables(function(result, err)
      respond(request_id, result, err)
    end)
    return true
  end
  if method == "stop_debugging" then
    dap_client.stop(function(result, err)
      respond(request_id, result, err)
    end)
    return true
  end
  if method == "pause_execution" then
    dap_client.pause(function(result, err)
      respond(request_id, result, err)
    end)
    return true
  end
  if method == "variable_children" then
    dap_client.variable_children(arguments.variables_reference, function(result, err)
      respond(request_id, result, err)
    end)
    return true
  end
  if method == "evaluate" then
    dap_client.evaluate(arguments.expression, function(result, err)
      respond(request_id, result, err)
    end)
    return true
  end
  local handlers = {
    start_debugging = function()
      local snapshot = require("dap-mcp").config_snapshot()
      arguments.auto_open_dap_ui = snapshot and snapshot.auto_open_dap_ui == true
      return dap_client.start(arguments)
    end,
    add_breakpoint = function()
      return dap_client.add_breakpoint(arguments)
    end,
    remove_breakpoint = function()
      return dap_client.remove_breakpoint(arguments)
    end,
    clear_all_breakpoints = dap_client.clear_breakpoints,
    list_breakpoints = dap_client.list_breakpoints,
  }
  local handler = handlers[method]
  if handler then
    local ok, result, err = pcall(handler)
    if not ok then
      respond(request_id, nil, errors.new("LUA_RPC_FAILURE", tostring(result)))
    else
      respond(request_id, result, err)
    end
    return true
  end
  local result, err = dap_client.action(method)
  respond(request_id, result, err)
  return true
end

return M
