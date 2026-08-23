local dap_mcp = require("dap-mcp")

if not dap_mcp.is_configured() then
  local configured, err = dap_mcp.setup()
  if not configured then
    vim.schedule(function()
      vim.notify(tostring(err), vim.log.levels.ERROR, { title = "dap-mcp.nvim" })
    end)
  end
end
