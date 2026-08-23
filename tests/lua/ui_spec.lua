local MiniTest = require("mini.test")

local expect = MiniTest.expect
local T = MiniTest.new_set()

T["missing dap-ui remains a no-op"] = function()
  package.loaded.dapui = nil
  package.preload.dapui = nil
  package.loaded["dap-mcp.ui"] = nil
  expect.equality(require("dap-mcp.ui").open(true), false)
end

T["installed dap-ui opens and closes exactly when owned"] = function()
  local calls = { open = 0, close = 0 }
  package.loaded.dapui = nil
  package.preload.dapui = function()
    return {
      open = function()
        calls.open = calls.open + 1
      end,
      close = function()
        calls.close = calls.close + 1
      end,
    }
  end
  package.loaded["dap-mcp.ui"] = nil
  local ui = require("dap-mcp.ui")

  expect.equality(ui.open(true), true)
  ui.close()
  ui.close()

  package.loaded.dapui = nil
  package.preload.dapui = nil
  expect.equality(calls.open, 1)
  expect.equality(calls.close, 1)
end

return T
