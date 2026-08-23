local root = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h:h:h")

vim.opt.runtimepath:prepend(root)
vim.opt.runtimepath:prepend(root .. "/deps/nvim-dap")
package.path = root .. "/lua/?.lua;" .. root .. "/lua/?/init.lua;" .. package.path

local dap = require("dap")
dap.adapters.codelldb = {
  type = "executable",
  command = vim.fn.expand("~/.local/share/nvim/mason/packages/codelldb/extension/adapter/codelldb"),
  name = "codelldb",
}

local configured, err = require("dap-mcp").setup({
  binary_path = root .. "/sidecar/target/debug/dap-mcp-sidecar",
  timeout_seconds = 8,
  shutdown_grace_ms = 3000,
  auto_open_dap_ui = false,
})
assert(configured, tostring(err))
