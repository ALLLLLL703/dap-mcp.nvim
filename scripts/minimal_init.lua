local root = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":p:h:h")

vim.opt.runtimepath:prepend(root)
vim.opt.runtimepath:prepend(root .. "/deps/mini.nvim")
vim.opt.runtimepath:prepend(root .. "/deps/nvim-dap")

package.path = root .. "/lua/?.lua;" .. root .. "/lua/?/init.lua;" .. package.path

require("mini.test").setup()
