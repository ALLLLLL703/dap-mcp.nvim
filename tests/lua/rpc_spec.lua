local MiniTest = require("mini.test")

local expect = MiniTest.expect
local T = MiniTest.new_set()

---Builds a minimal nvim-dap listener registry.
---@return table
local function fake_dap()
  return {
    listeners = {
      after = {
        event_initialized = {},
        event_stopped = {},
        event_continued = {},
        event_output = {},
        event_terminated = {},
        event_exited = {},
      },
    },
    session = function()
      return nil
    end,
  }
end

T["register forwards DAP lifecycle events"] = function()
  local dap = fake_dap()
  local notifications = {}
  package.loaded.dap = dap
  package.loaded["dap-mcp.dap_events"] = nil
  package.loaded["dap-mcp.rpc"] = nil
  local original_rpcnotify = vim.rpcnotify
  vim.rpcnotify = function(...)
    table.insert(notifications, { ... })
  end

  require("dap-mcp.rpc").register(41)
  dap.listeners.after.event_initialized["dap-mcp.nvim"]({ config = { name = "Launch app" } })
  dap.listeners.after.event_stopped["dap-mcp.nvim"](
    { current_frame = { id = 9 } },
    { threadId = 3 }
  )

  vim.rpcnotify = original_rpcnotify
  expect.equality(notifications[1][1], 41)
  expect.equality(notifications[1][2], "dap_mcp_event")
  expect.equality(notifications[1][3].configuration_name, "Launch app")
  expect.equality(notifications[2][3].frame_id, 9)
  expect.equality(notifications[2][3].thread_id, 3)
end

T["dispatch returns stable unknown-action error"] = function()
  local dap = fake_dap()
  local notifications = {}
  package.loaded.dap = dap
  package.loaded["dap-mcp.dap_events"] = nil
  package.loaded["dap-mcp.dap_client"] = nil
  package.loaded["dap-mcp.rpc"] = nil
  local original_rpcnotify = vim.rpcnotify
  vim.rpcnotify = function(...)
    table.insert(notifications, { ... })
  end

  local rpc = require("dap-mcp.rpc")
  rpc.register(42)
  rpc.dispatch("not_a_method", 7, {})

  vim.rpcnotify = original_rpcnotify
  local envelope = notifications[#notifications][4]
  expect.equality(envelope.ok, false)
  expect.equality(envelope.error.code, "NO_ACTIVE_SESSION")
end

return T
