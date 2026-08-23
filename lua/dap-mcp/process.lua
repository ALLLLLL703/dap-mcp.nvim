local errors = require("dap-mcp.errors")

local M = {}

---@class DapMcpProcessCallbacks
---@field on_exit fun(exit_code: integer)
---@field on_stderr fun(lines: string[])

---Checks whether a sidecar binary is executable.
---@param binary_path string
---@return boolean
function M.is_executable(binary_path)
  return vim.fn.executable(binary_path) == 1
end

---Spawns the sidecar as a Neovim RPC child.
---@param config DapMcpConfig
---@param port integer
---@param callbacks DapMcpProcessCallbacks
---@return integer? channel
---@return DapMcpError? error
function M.spawn(config, port, callbacks)
  local command = {
    config.binary_path,
    "--port",
    tostring(port),
    "--host",
    config.bind_host,
    "--timeout-seconds",
    tostring(config.timeout_seconds),
    "--log-level",
    config.log_level,
  }
  if config.allow_external_files then
    table.insert(command, "--allow-external-files")
  end
  local channel = vim.fn.jobstart(command, {
    rpc = true,
    stderr_buffered = true,
    on_exit = function(_, exit_code)
      callbacks.on_exit(exit_code)
    end,
    on_stderr = function(_, lines)
      callbacks.on_stderr(lines)
    end,
  })
  if channel <= 0 then
    return nil,
      errors.new("SIDECAR_SPAWN_FAILED", "Failed to start the sidecar process", {
        result = channel,
      })
  end
  return channel, nil
end

---Requests graceful shutdown and schedules a forced-stop fallback.
---@param channel integer
---@param grace_ms integer
---@return nil
function M.request_shutdown(channel, grace_ms)
  local called, sent = pcall(vim.rpcnotify, channel, "dap_mcp_shutdown")
  if not called or sent == false then
    vim.notify("[sidecar.shutdown_notify_failed] Failed to notify the sidecar", vim.log.levels.WARN)
  end
  vim.defer_fn(function()
    if vim.fn.jobwait({ channel }, 0)[1] == -1 then
      vim.fn.jobstop(channel)
    end
  end, grace_ms)
end

return M
