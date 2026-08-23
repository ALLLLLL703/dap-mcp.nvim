---@class DapMcpLogger
---@field private minimum integer
---@field private sink fun(level: integer, message: string)
local Logger = {}
Logger.__index = Logger

local LEVELS = { debug = 10, info = 20, warn = 30, error = 40 }
local VIM_LEVELS = {
  debug = vim.log.levels.DEBUG,
  info = vim.log.levels.INFO,
  warn = vim.log.levels.WARN,
  error = vim.log.levels.ERROR,
}

---Creates a configured logger.
---@param level DapMcpLogLevel
---@param sink? fun(level: integer, message: string)
---@return DapMcpLogger
function Logger.new(level, sink)
  return setmetatable({
    minimum = LEVELS[level],
    sink = sink or function(vim_level, message)
      vim.notify(message, vim_level, { title = "dap-mcp.nvim" })
    end,
  }, Logger)
end

---Writes one log event when its level is enabled.
---@param level DapMcpLogLevel
---@param event string
---@param message string
---@return nil
function Logger:log(level, event, message)
  if LEVELS[level] < self.minimum then
    return
  end
  self.sink(VIM_LEVELS[level], string.format("[%s] %s", event, message))
end

local M = {}

---Creates a configured logger facade.
---@param level DapMcpLogLevel
---@param sink? fun(level: integer, message: string)
---@return DapMcpLogger
function M.new(level, sink)
  return Logger.new(level, sink)
end

return M
