local MiniTest = require("mini.test")

local expect = MiniTest.expect
local T = MiniTest.new_set()

T["setup exposes a stable configured snapshot"] = function()
  package.loaded["dap-mcp"] = nil
  local plugin = require("dap-mcp")
  local configured = plugin.setup({ binary_path = "/tmp/custom-sidecar" })

  expect.equality(configured, true)
  expect.equality(plugin.is_configured(), true)
  expect.equality(plugin.config_snapshot().binary_path, "/tmp/custom-sidecar")
end

return T
