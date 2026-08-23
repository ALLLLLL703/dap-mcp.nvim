local MiniTest = require("mini.test")

local expect = MiniTest.expect
local T = MiniTest.new_set()

---Reloads the DAP facade against one fake nvim-dap module.
---@param dap table
---@return table
local function client_with(dap)
  package.loaded.dap = dap
  package.loaded["dap-mcp.dap_client"] = nil
  return require("dap-mcp.dap_client")
end

T["stop responds after nvim-dap on_done"] = function()
  local response
  local dap = {
    session = function()
      return {}
    end,
  }
  dap.terminate = function(options)
    expect.equality(response, nil)
    options.on_done()
  end

  client_with(dap).stop(function(result)
    response = result
  end)

  expect.equality(response.accepted, true)
end

T["pause selects an adapter thread without interactive UI"] = function()
  local selected_thread
  local session = {
    request = function(_, command, _, callback)
      expect.equality(command, "threads")
      callback(nil, { threads = { { id = 17 }, { id = 18 } } })
    end,
  }
  local dap = {
    session = function()
      return session
    end,
    pause = function(thread_id)
      selected_thread = thread_id
    end,
  }

  client_with(dap).pause(function(result)
    expect.equality(result.accepted, true)
  end)

  expect.equality(selected_thread, 17)
end

return T
