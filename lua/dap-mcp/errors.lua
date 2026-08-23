---@class DapMcpError
---@field code string Stable machine-readable error code.
---@field message string Actionable English error message.
---@field context table<string, unknown>
local Error = {}
Error.__index = Error

---Creates a structured plugin error.
---@param code string
---@param message string
---@param context? table<string, unknown>
---@return DapMcpError
function Error.new(code, message, context)
  return setmetatable({
    code = code,
    message = message,
    context = context or {},
  }, Error)
end

---Formats an error for user-facing notifications.
---@return string
function Error:__tostring()
  return string.format("[%s] %s", self.code, self.message)
end

local M = {}

---Creates a structured plugin error.
---@param code string
---@param message string
---@param context? table<string, unknown>
---@return DapMcpError
function M.new(code, message, context)
  return Error.new(code, message, context)
end

---Checks whether a value is a structured plugin error.
---@param value unknown
---@return boolean
function M.is(value)
  return getmetatable(value) == Error
end

return M
