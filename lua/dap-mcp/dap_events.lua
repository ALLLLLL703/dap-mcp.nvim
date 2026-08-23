local M = {}

local LISTENER_KEY = "dap-mcp.nvim"

---Sends one DAP lifecycle event to the registered sidecar channel.
---@param channel integer
---@param event table<string, unknown>
---@return nil
local function notify(channel, event)
  vim.rpcnotify(channel, "dap_mcp_event", event)
end

---Installs idempotent nvim-dap listeners for the active sidecar channel.
---@param channel integer
---@return nil
function M.register(channel)
  local dap = require("dap")
  dap.listeners.after.event_initialized[LISTENER_KEY] = function(session)
    notify(channel, {
      kind = "session_started",
      configuration_name = (session.config or {}).name or "unnamed",
    })
  end
  dap.listeners.after.event_stopped[LISTENER_KEY] = function(session, body)
    notify(channel, {
      kind = "stopped",
      thread_id = body and body.threadId or nil,
      frame_id = session.current_frame and session.current_frame.id or nil,
    })
  end
  dap.listeners.after.event_continued[LISTENER_KEY] = function()
    notify(channel, { kind = "continued" })
  end
  dap.listeners.after.event_output[LISTENER_KEY] = function(_, body)
    notify(channel, {
      kind = "output",
      category = body and body.category or nil,
      output = body and body.output or "",
    })
  end
  local terminated = function()
    notify(channel, { kind = "terminated" })
  end
  dap.listeners.after.event_terminated[LISTENER_KEY] = terminated
  dap.listeners.after.event_exited[LISTENER_KEY] = terminated
end

return M
