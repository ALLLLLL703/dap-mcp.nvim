local MiniTest = require("mini.test")
local config = require("dap-mcp.config")

local expect = MiniTest.expect
local T = MiniTest.new_set()

T["resolve uses safe defaults"] = function()
  local result, err = config.resolve()

  expect.equality(err, nil)
  expect.equality(result.bind_host, "127.0.0.1")
  expect.equality(result.timeout_seconds, 180)
  expect.equality(result.allow_external_files, false)
  expect.equality(result.auto_open_dap_ui, true)
end

T["resolve rejects non-loopback hosts"] = function()
  local result, err = config.resolve({ bind_host = "0.0.0.0" })

  expect.equality(result, nil)
  expect.equality(err.code, "INVALID_BIND_HOST")
end

T["resolve rejects non-positive and fractional timeouts"] = MiniTest.new_set({
  parametrize = { { 0 }, { -1 }, { 1.5 } },
}, {
  test = function(timeout)
    local result, err = config.resolve({ timeout_seconds = timeout })

    expect.equality(result, nil)
    expect.equality(err.code, "INVALID_TIMEOUT")
  end,
})

T["resolve returns an isolated snapshot"] = function()
  local options = { log_level = "debug" }
  local result = config.resolve(options)
  options.log_level = "error"

  expect.equality(result.log_level, "debug")
end

return T
