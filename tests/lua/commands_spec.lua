local MiniTest = require("mini.test")
local dap_mcp = require("dap-mcp")

local expect = MiniTest.expect
local T = MiniTest.new_set()

T["setup registers commands without starting the server"] = function()
  local configured, err = dap_mcp.setup({ binary_path = "/bin/false" })

  expect.equality(configured, true)
  expect.equality(err, nil)
  expect.equality(vim.fn.exists(":DebuggerMcpStart"), 2)
  expect.equality(vim.fn.exists(":DebuggerMcpStop"), 2)
  expect.equality(dap_mcp.state(), "stopped")
end

T["invalid command port reports a stable error"] = function()
  local notifications = {}
  local original_notify = vim.notify

  ---Captures one command notification.
  ---@param message string
  ---@return nil
  local function capture_notification(message)
    table.insert(notifications, message)
  end
  vim.notify = capture_notification
  dap_mcp.setup({ binary_path = "/bin/false" })

  vim.cmd("DebuggerMcpStart invalid")
  vim.notify = original_notify

  expect.equality(#notifications, 1)
  expect.equality(notifications[1]:find("INVALID_PORT", 1, true) ~= nil, true)
  expect.equality(dap_mcp.state(), "stopped")
end

return T
