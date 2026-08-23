# dap-mcp.nvim 产品实施方案

## 行为目标

实现一个默认关闭、由 `:DebuggerMcpStart <port>` 手动开启的 Neovim debugger MCP。Rust sidecar 提供 loopback-only Streamable HTTP `/mcp`，通过父进程 Msgpack-RPC 驱动当前 Neovim 中的 `nvim-dap`。首版实现已确认的 16 个 DebugMCP-compatible tools，并从 `<workingDirectory>/.vscode/launch.json` 精确选择具名 launch/attach configuration。

## 模块与责任

### Lua plugin

- `plugin/dap-mcp.lua`：只注册 Start/Stop 命令。
- `lua/dap-mcp/init.lua`：`setup(opts)` composition root。
- `config.lua`：raw config 验证与 immutable snapshot。
- `lifecycle.lua`：sidecar spawn/stop/crash 与显式 state machine。
- `dap_client.lua`：窄化 `nvim-dap` API，管理单活动 session、breakpoint、step、variables 与 evaluate。
- `dap_events.lua`：监听 DAP event，通过 `vim.rpcnotify()` 通知 sidecar。
- `launch.lua`：接收 Rust 解析后的 configuration 并调用 `dap.run(config)`。
- `ui.lua`：可选 `nvim-dap-ui` 自动开关。
- `errors.lua`、`logger.lua`：稳定 error code 与统一英文日志。

### Rust sidecar

- `main.rs`：composition root、nvim-rs parent connection、server lifecycle。
- `config.rs`、`error.rs`、`logging.rs`：validated config、typed error、tracing。
- `launch/`：jsonc-parser 读取、schema 验证、name 选择与 workspace containment。
- `security/`：loopback bind 与 Host/Origin/port validation。
- `state/`：single debug session 与 tool-call state。
- `nvim/`：Msgpack-RPC client、notification handler 与 DTO mapping。
- `mcp/`：rmcp Streamable HTTP transport、schemas 和 16 个 tool handlers。

## 实现顺序

1. 建立 Cargo/Lua/test/formatter/linter 骨架并锁定已批准依赖。
2. 先写 pure config/state/validation tests，再实现 Lua lifecycle 与 Rust launch/security/state。
3. 接通 nvim-rs parent RPC 与 Lua DAP facade，使用事件而非 polling/busy-wait。
4. 注册 MCP schemas 和 tools，统一 timeout、structured error 与 graceful shutdown。
5. 使用 mini.test、cargo test、headless integration、CodeLLDB fixture 与 neovim-mcp 逐层验证。

## 外部依赖决策

- `rmcp`：官方 Rust MCP SDK，代替自制 Streamable HTTP/MCP protocol。
- `nvim-rs`：提供 `create::new_parent()`，代替自制 Msgpack framing。
- `jsonc-parser`：正确处理 VS Code JSONC comments/trailing commas，代替字符串剥离。
- `serde`/`serde_json`：稳定 DTO 与 JSON boundary。
- `tokio`/`tracing`：async runtime、timeout、cancellation 与 structured logs。
- Lua 只依赖必需的 `nvim-dap`；`nvim-dap-ui` 保持 optional；测试使用 `mini.test`。

所有 Cargo dependency 使用受控 semver 并提交 Cargo.lock。实现前通过 Context7/GitHub 源码核对当前 API；若 API 与此方案冲突，先更新本文件和 `todo.md`，不静默换架构。

## Test seams

- config、port、workspace containment、JSONC selection、Host/Origin 和 state transition 为 pure unit tests。
- process launcher、clock/timeout、filesystem、Neovim RPC 和 nvim-dap 通过窄 collaborator 或 facade 替换。
- Msgpack-RPC 与 Streamable HTTP 使用真实 integration path。
- `tests/fixtures/rust-codelldb/` 验证 launch/attach、breakpoint、step、variables、evaluate、restart、stop。

## 完成门禁

- 所有手写 Lua/Rust/test 文件不超过 400 行；所有 named function/type 有 doc，Lua 使用 EmmyLua。
- StyLua、Selene、LuaLS、rustfmt、Clippy `-D warnings`、mini.test、cargo test 全部通过。
- 用户流程通过 neovim-mcp 完整验收；复杂跨进程失败按需通过 debugger-mcp 定位。
- 每完成一个 `todo.md` 阶段立即勾选并附命令、场景与结果。
