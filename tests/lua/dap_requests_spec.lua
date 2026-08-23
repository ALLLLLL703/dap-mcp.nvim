local MiniTest = require("mini.test")
local requests = require("dap-mcp.dap_requests")

local expect = MiniTest.expect
local T = MiniTest.new_set()

T["evaluate requires a stopped frame"] = function()
  local received
  requests.evaluate(
    {
      session = function()
        return {}
      end,
    },
    "value",
    function(_, err)
      received = err
    end
  )
  expect.equality(received.code, "NO_ACTIVE_FRAME")
end

T["evaluate maps the active frame and response"] = function()
  local received
  local session = {
    current_frame = { id = 12 },
    request = function(_, command, arguments, callback)
      expect.equality(command, "evaluate")
      expect.equality(arguments.frameId, 12)
      callback(nil, { result = "42", variablesReference = 0 })
    end,
  }
  requests.evaluate(
    {
      session = function()
        return session
      end,
    },
    "6 * 7",
    function(result)
      received = result
    end
  )
  expect.equality(received.result, "42")
end

T["variables merges non-expensive scopes"] = function()
  local received
  local session = { current_frame = { id = 5 } }
  session.request = function(_, command, arguments, callback)
    if command == "scopes" then
      callback(nil, {
        scopes = {
          { variablesReference = 10, expensive = false },
          { variablesReference = 20, expensive = true },
        },
      })
    else
      expect.equality(arguments.variablesReference, 10)
      callback(nil, { variables = { { name = "answer", value = "42" } } })
    end
  end
  requests.variables({
    session = function()
      return session
    end,
  }, function(result)
    received = result
  end)
  expect.equality(#received, 1)
  expect.equality(received[1].name, "answer")
end

T["children requests one exact variables reference"] = function()
  local received
  local session = {
    request = function(_, command, arguments, callback)
      expect.equality(command, "variables")
      expect.equality(arguments.variablesReference, 77)
      callback(nil, { variables = { { name = "child", value = "1" } } })
    end,
  }
  requests.children(
    {
      session = function()
        return session
      end,
    },
    77,
    function(result)
      received = result
    end
  )
  expect.equality(received[1].name, "child")
end

return T
