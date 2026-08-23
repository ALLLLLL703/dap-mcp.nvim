local errors = require("dap-mcp.errors")

---@alias DapMcpLogLevel "debug"|"info"|"warn"|"error"

---@class DapMcpConfig
---@field binary_path string
---@field bind_host "localhost"|"127.0.0.1"|"::1"
---@field timeout_seconds integer
---@field shutdown_grace_ms integer
---@field log_level DapMcpLogLevel
---@field allow_external_files boolean
---@field auto_open_dap_ui boolean

local M = {}

local ALLOWED_HOSTS = { ["localhost"] = true, ["127.0.0.1"] = true, ["::1"] = true }
local ALLOWED_LEVELS = { debug = true, info = true, warn = true, error = true }

---Returns the repository root derived from this module path.
---@return string
local function repository_root()
  local source = debug.getinfo(1, "S").source:sub(2)
  return vim.fn.fnamemodify(source, ":p:h:h:h")
end

---Returns a fresh default configuration.
---@return DapMcpConfig
local function defaults()
  return {
    binary_path = repository_root() .. "/sidecar/target/release/dap-mcp-sidecar",
    bind_host = "127.0.0.1",
    timeout_seconds = 180,
    shutdown_grace_ms = 1000,
    log_level = "info",
    allow_external_files = false,
    auto_open_dap_ui = true,
  }
end

---Validates a merged configuration.
---@param config DapMcpConfig
---@return DapMcpConfig? config
---@return DapMcpError? error
local function validate(config)
  if type(config.binary_path) ~= "string" or config.binary_path == "" then
    return nil, errors.new("INVALID_BINARY_PATH", "binary_path must be a non-empty string")
  end
  if not ALLOWED_HOSTS[config.bind_host] then
    return nil, errors.new("INVALID_BIND_HOST", "bind_host must be a loopback address")
  end
  if
    type(config.timeout_seconds) ~= "number"
    or config.timeout_seconds % 1 ~= 0
    or config.timeout_seconds <= 0
  then
    return nil, errors.new("INVALID_TIMEOUT", "timeout_seconds must be a positive integer")
  end
  if
    type(config.shutdown_grace_ms) ~= "number"
    or config.shutdown_grace_ms % 1 ~= 0
    or config.shutdown_grace_ms <= 0
  then
    return nil, errors.new("INVALID_SHUTDOWN_GRACE", "shutdown_grace_ms must be a positive integer")
  end
  if not ALLOWED_LEVELS[config.log_level] then
    return nil, errors.new("INVALID_LOG_LEVEL", "log_level must be debug, info, warn, or error")
  end
  if type(config.allow_external_files) ~= "boolean" then
    return nil, errors.new("INVALID_EXTERNAL_FILE_POLICY", "allow_external_files must be boolean")
  end
  if type(config.auto_open_dap_ui) ~= "boolean" then
    return nil, errors.new("INVALID_DAP_UI_POLICY", "auto_open_dap_ui must be boolean")
  end
  return vim.deepcopy(config), nil
end

---Merges and validates user configuration.
---@param options? table<string, unknown>
---@return DapMcpConfig? config
---@return DapMcpError? error
function M.resolve(options)
  if options ~= nil and type(options) ~= "table" then
    return nil, errors.new("INVALID_CONFIG", "setup options must be a table")
  end
  local config = vim.tbl_deep_extend("force", defaults(), options or {})
  return validate(config)
end

return M
