local MiniTest = require("mini.test")
local lifecycle = require("dap-mcp.lifecycle")

local expect = MiniTest.expect
local T = MiniTest.new_set()

---Creates deterministic lifecycle collaborators for a test.
---@param overrides? table<string, unknown>
---@return DapMcpLifecycleDeps
---@return table<string, unknown>
local function make_deps(overrides)
  local calls = { stop_debugging = 0, close_ui = 0, shutdown = 0 }
  local deps = {
    is_dap_available = function()
      return true
    end,
    is_executable = function()
      return true
    end,
    spawn = function(_, _, callbacks)
      calls.callbacks = callbacks
      return 12, nil
    end,
    stop_debugging = function()
      calls.stop_debugging = calls.stop_debugging + 1
    end,
    close_ui = function()
      calls.close_ui = calls.close_ui + 1
    end,
    request_shutdown = function(channel, grace_ms)
      calls.shutdown = calls.shutdown + 1
      calls.channel = channel
      calls.grace_ms = grace_ms
    end,
  }
  return vim.tbl_extend("force", deps, overrides or {}), calls
end

---Creates a no-op test logger.
---@return DapMcpLogger
local function make_logger()
  return { log = function() end }
end

---Returns the minimal validated config used by lifecycle tests.
---@return DapMcpConfig
local function make_config()
  return { binary_path = "/tmp/sidecar", shutdown_grace_ms = 250 }
end

T["start validates port boundaries"] = MiniTest.new_set({
  parametrize = { { 0 }, { 65536 }, { 1.5 }, { "3001" } },
}, {
  test = function(port)
    local deps = make_deps()
    local runtime = lifecycle.new(make_config(), deps, make_logger())
    local started, err = runtime:start(port)

    expect.equality(started, false)
    expect.equality(err.code, "INVALID_PORT")
    expect.equality(runtime:state(), "stopped")
  end,
})

T["start rejects a missing nvim-dap dependency"] = function()
  local deps = make_deps({
    is_dap_available = function()
      return false
    end,
  })
  local runtime = lifecycle.new(make_config(), deps, make_logger())
  local started, err = runtime:start(3001)

  expect.equality(started, false)
  expect.equality(err.code, "NVIM_DAP_MISSING")
end

T["start reaches running and rejects duplicate starts"] = function()
  local deps = make_deps()
  local runtime = lifecycle.new(make_config(), deps, make_logger())

  expect.equality(runtime:start(3001), true)
  expect.equality(runtime:state(), "running")
  expect.equality(runtime:active_channel(), 12)
  local started, err = runtime:start(3002)
  expect.equality(started, false)
  expect.equality(err.code, "INVALID_LIFECYCLE_TRANSITION")
end

T["stop orders boundary cleanup and exit reaches stopped"] = function()
  local deps, calls = make_deps()
  local runtime = lifecycle.new(make_config(), deps, make_logger())
  runtime:start(3001)

  expect.equality(runtime:stop(), true)
  expect.equality(runtime:state(), "stopping")
  expect.equality(calls.stop_debugging, 1)
  expect.equality(calls.close_ui, 1)
  expect.equality(calls.channel, 12)
  expect.equality(calls.grace_ms, 250)
  calls.callbacks.on_exit(0)
  expect.equality(runtime:state(), "stopped")
end

T["unexpected sidecar exit reaches failed without restart"] = function()
  local deps, calls = make_deps()
  local runtime = lifecycle.new(make_config(), deps, make_logger())
  runtime:start(3001)

  calls.callbacks.on_exit(7)
  expect.equality(runtime:state(), "failed")
  expect.equality(runtime:active_channel(), nil)
end

return T
