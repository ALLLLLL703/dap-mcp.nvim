# dap-mcp.nvim 初始实现清单

- [x] 阶段 1：建立 Lua/Rust 工程骨架、配置与质量工具，锁定依赖并通过基础检查。
  - 证据：Context7 + crates.io/GitHub 核对 API；Cargo.lock 生成；`cargo check --manifest-path sidecar/Cargo.toml` 通过；Neovim 0.12.4、Rust 1.90、StyLua、LuaLS 环境确认。
  - 依赖：采用 rmcp 3.1.4、jsonc-parser 0.33.1、clap/Tokio/Serde/Tracing；用户明确批准 nvim-rs 0.9.2 LGPL-3.0 例外。
- [x] 阶段 2：实现并测试 Lua 配置、错误、日志、命令与 sidecar 生命周期状态机。
  - 证据：`mini.test` 运行 config 6、lifecycle 8、commands 2 个场景全部通过；`stylua --check` 通过；LuaLS absolute config check 零诊断；所有 Lua/test 文件少于 400 行。
- [x] 阶段 3：实现并测试 Rust 配置、结构化错误、JSONC launch 选择、HTTP 安全与 debug state。
  - 证据：12 个 Rust tests 覆盖 config、JSONC comments/trailing commas、missing/duplicate name、workspace containment、Host/Origin/port、single-session transitions；`cargo fmt --check` 与 Clippy `-D warnings` 通过。
  - 修复记录：首次启用 jsonc-parser feature 不完整，补 `serde`；IPv6 Origin parser 返回 `[::1]`，扩展明确 allow-list 后全量重跑通过。
- [x] 阶段 4：实现并测试 Neovim Msgpack-RPC、nvim-dap 操作、事件桥与可选 dap-ui。
  - 证据：Lua RPC 使用 request-id + `vim.rpcnotify` 异步 envelope，避免 Neovim main loop deadlock；覆盖 launch、step/control、breakpoint、scope/variables/evaluate、DAP lifecycle event 与 dap-ui。
  - 验证：21 个 mini.test 场景通过；LuaLS 零诊断；Rust bridge/handler 纳入 14 个 `cargo test`，rustfmt 与 Clippy `-D warnings` 通过；所有源码与测试文件少于 400 行。
- [x] 阶段 5：实现 16 个 MCP tools、Streamable HTTP server、超时与优雅关闭，并完成跨边界测试。
  - 证据：rmcp 3.1.4 注册全部 16 个上游兼容 tool name 与 camelCase schema；stateful `/mcp`、Host/Origin/port 防护、180s operation timeout、210s backstop、1..50 variable names、100 descendants 与 session cancellation 已实现；首版没有 memory-read tool，因此 4096-byte memory cap 没有可调用表面。
  - 跨边界：真实 headless Neovim 以 RPC child 启动 sidecar，HTTP initialize、tools/list、`list_breakpoints` Lua↔Rust round-trip 成功；恶意 Host 返回 403；关闭问题经复现修复后 sidecar exit code 从 143 变为 0。
  - 验证：22 个 mini.test、17 个 Rust tests、LuaLS、StyLua、rustfmt、Clippy `-D warnings` 全部通过。
- [x] 阶段 6：加入 Rust+CodeLLDB fixture，运行 formatter/lint/unit/integration/build，并通过 neovim-mcp 真实验收。
  - CodeLLDB launch E2E：JSONC launch.json、`${workspaceFolder}`、breakpoint hit、list/get variables、evaluate、step_over/into/out、continue、pause、restart、stop、list/clear breakpoint 全部真实通过。
  - CodeLLDB attach E2E：fixture 使用 process-local `PR_SET_PTRACER_ANY` 保持系统 Yama policy 不变；具名 attach configuration、初始化与修复后的 callback-based stop 全部通过。
  - neovim-mcp：连接健康；真实 Neovim 中 setup snapshot 正确，`:DebuggerMcpStart 39013` 后 state 为 running、HTTP listener 响应，`:DebuggerMcpStop` 后 state 为 stopped。
  - 最终门禁：27 个 mini.test scenarios、20 个 Rust tests、fixture cargo test/build、StyLua、Selene 0.31.0、LuaLS、rustfmt、Clippy `-D warnings`、release build、AGENTS 全引用与每文件 `<400` 行全部通过。
- [x] 同步更新 `agent-instructions/Project.md` 的 implemented 状态与最终用户文档。
  - 证据：`Project.md` 已改为当前实际模块拓扑；新增 `README.md` 覆盖 build/setup、adapter、Start/Stop、16 tools、安全边界、launch.json 兼容与验证命令。

## Verification

- `cargo test --all-targets --all-features`：20 passed；`cargo clippy --all-targets --all-features -- -D warnings`、`cargo fmt --check`、`cargo build --release`：passed。
- `mini.test`：27 passed；StyLua、Selene、LuaLS：0 errors / 0 warnings / 0 diagnostics。
- HTTP/MCP：initialize、session reuse、tools/list、真实 tool call、Host 403、安全 loopback 与 graceful shutdown exit 0：passed。
- DAP：真实 CodeLLDB launch 和 attach、控制流、变量、表达式、breakpoint 与清理：passed。
- 失败与修复证据：修复 user setup 被 plugin bootstrap 覆盖、Rust struct 误编码为 Msgpack array、`${workspaceFolder}` 错误根目录、shutdown exit 143、attach stop event 缺失、pause 线程选择器与 Yama attach policy；每项均补 regression 或真实场景重跑。
- 剩余 blocker：无。
