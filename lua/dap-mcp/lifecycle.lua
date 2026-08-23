local errors = require("dap-mcp.errors")

---@alias DapMcpLifecycleState "stopped"|"starting"|"running"|"stopping"|"failed"

---@class DapMcpLifecycleDeps
---@field is_dap_available fun(): boolean
---@field is_executable fun(path: string): boolean
---@field spawn fun(config: DapMcpConfig, port: integer, callbacks: DapMcpProcessCallbacks): integer?, DapMcpError?
---@field stop_debugging fun(): nil
---@field close_ui fun(): nil
---@field request_shutdown fun(channel: integer, grace_ms: integer): nil

---@class DapMcpLifecycle
---@field private config DapMcpConfig
---@field private deps DapMcpLifecycleDeps
---@field private logger DapMcpLogger
---@field private current_state DapMcpLifecycleState
---@field private channel integer?
local Lifecycle = {}
Lifecycle.__index = Lifecycle

---Checks whether a port is a valid non-zero TCP port.
---@param port unknown
---@return boolean
local function is_valid_port(port)
  return type(port) == "number" and port % 1 == 0 and port >= 1 and port <= 65535
end

---Creates a lifecycle state machine.
---@param config DapMcpConfig
---@param deps DapMcpLifecycleDeps
---@param logger DapMcpLogger
---@return DapMcpLifecycle
function Lifecycle.new(config, deps, logger)
  return setmetatable({
    config = vim.deepcopy(config),
    deps = deps,
    logger = logger,
    current_state = "stopped",
    channel = nil,
  }, Lifecycle)
end

---Returns the current runtime state.
---@return DapMcpLifecycleState
function Lifecycle:state()
  return self.current_state
end

---Returns the active sidecar RPC channel.
---@return integer?
function Lifecycle:active_channel()
  return self.channel
end

---Handles a sidecar process exit.
---@param exit_code integer
---@return nil
function Lifecycle:on_exit(exit_code)
  local expected = self.current_state == "stopping"
  self.channel = nil
  self.current_state = expected and "stopped" or "failed"
  local level = expected and "info" or "error"
  self.logger:log(level, "sidecar.exited", string.format("Sidecar exited with code %d", exit_code))
end

---Handles buffered sidecar stderr without exposing it as a user notification at info level.
---@param lines string[]
---@return nil
function Lifecycle:on_stderr(lines)
  local message = table.concat(
    vim.tbl_filter(function(line)
      return line ~= ""
    end, lines),
    "\n"
  )
  if message ~= "" then
    self.logger:log("debug", "sidecar.stderr", message)
  end
end

---Starts the sidecar for one validated port.
---@param port unknown
---@return boolean started
---@return DapMcpError? error
function Lifecycle:start(port)
  if self.current_state ~= "stopped" and self.current_state ~= "failed" then
    return false, errors.new("INVALID_LIFECYCLE_TRANSITION", "The sidecar is already active")
  end
  if not is_valid_port(port) then
    return false, errors.new("INVALID_PORT", "port must be an integer from 1 to 65535")
  end
  if not self.deps.is_dap_available() then
    return false,
      errors.new("NVIM_DAP_MISSING", "nvim-dap is required before starting dap-mcp.nvim")
  end
  if not self.deps.is_executable(self.config.binary_path) then
    return false,
      errors.new("SIDECAR_NOT_EXECUTABLE", "The configured sidecar binary is not executable")
  end

  self.current_state = "starting"
  local channel, spawn_error = self.deps.spawn(self.config, port, {
    on_exit = function(exit_code)
      self:on_exit(exit_code)
    end,
    on_stderr = function(lines)
      self:on_stderr(lines)
    end,
  })
  if not channel then
    self.current_state = "failed"
    return false, spawn_error
  end
  self.channel = channel
  self.current_state = "running"
  self.logger:log("info", "sidecar.started", string.format("MCP server started on port %d", port))
  return true, nil
end

---Begins an orderly shutdown of DAP, UI, MCP and sidecar resources.
---@return boolean stopping
---@return DapMcpError? error
function Lifecycle:stop()
  if self.current_state ~= "running" or not self.channel then
    return false, errors.new("SIDECAR_NOT_RUNNING", "The sidecar is not running")
  end
  self.current_state = "stopping"
  self.deps.stop_debugging()
  self.deps.close_ui()
  self.deps.request_shutdown(self.channel, self.config.shutdown_grace_ms)
  self.logger:log("info", "sidecar.stopping", "Sidecar shutdown requested")
  return true, nil
end

local M = {}

---Creates a lifecycle state machine.
---@param config DapMcpConfig
---@param deps DapMcpLifecycleDeps
---@param logger DapMcpLogger
---@return DapMcpLifecycle
function M.new(config, deps, logger)
  return Lifecycle.new(config, deps, logger)
end

return M
