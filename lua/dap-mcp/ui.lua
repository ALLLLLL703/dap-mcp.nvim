local M = {}

local opened_by_plugin = false

---Loads the optional nvim-dap-ui dependency.
---@return table? dapui
local function load_ui()
  -- nvim-dap-ui is optional and must not become a startup dependency.
  local available, dapui = pcall(require, "dapui")
  return available and dapui or nil
end

---Opens nvim-dap-ui when configured and installed.
---@param enabled boolean
---@return boolean opened
function M.open(enabled)
  if not enabled then
    return false
  end
  local dapui = load_ui()
  if not dapui then
    return false
  end
  dapui.open()
  opened_by_plugin = true
  return true
end

---Closes nvim-dap-ui only when this plugin opened it.
---@return nil
function M.close()
  if not opened_by_plugin then
    return
  end
  local dapui = load_ui()
  if dapui then
    dapui.close()
  end
  opened_by_plugin = false
end

return M
