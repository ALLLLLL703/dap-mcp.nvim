# Test 与 Debug

## 完成门禁

没有通过相关真实场景测试，就不能宣称代码完成。static reasoning、LSP diagnostics、type check 或“应该能工作”都不能单独作为完成证据。

每次代码变更必须：

1. 明确受影响的真实使用场景。
2. 选择能证明这些场景的最强实际验证方式。
3. 补充缺失的 regression test。
4. 执行测试或真实程序验证。
5. 修复失败并重复执行，直到全部相关场景通过。
6. 明确报告未测试场景为 blocker，不得静默跳过。

## 测试框架

- Lua 与 Neovim 内测试使用 `mini.test`。
- Rust unit/integration test 使用内建 `cargo test`。
- Rust DAP E2E fixture 使用 CodeLLDB 调试真实小型 Rust program。
- 不新增第二套同层测试框架，除非 `$grill-me` 后确认迁移理由和退役计划。

## 分层策略

### 每次相关代码变更

- 始终运行受影响的 fast unit tests。
- Lua parse/config/state 逻辑通过 `mini.test` 验证。
- Rust schema/security/JSONC/state/error 逻辑通过 focused `cargo test` 验证。
- 运行对应 formatter、linter、type diagnostics 和 compiler check。

### 跨 Lua/Rust 或外部边界变化

增加 integration test，至少覆盖：

- Neovim `rpc=true` 启动 sidecar 与 `nvim-rs::create::new_parent()` handshake。
- request、response、notification、timeout、cancellation 和 disconnect。
- Start/Stop state transition 与异常退出清理。
- Streamable HTTP initialize、session reuse、DELETE/shutdown 和 malformed request。
- Host、Origin、port、loopback bind 与占用端口拒绝。

### 用户流程或 DAP 行为变化

除自动测试外，必须通过 `neovim-mcp` 驱动真实 Neovim，完成本文件的体感验收流程。

## 关键分支覆盖

不设置全局行覆盖率百分比。以下模块必须覆盖 success、failure 和 boundary branch：

- lifecycle state machine 与非法 transition。
- port validation、occupied port 和 sidecar crash。
- Host/Origin/port validation 与 loopback restriction。
- JSONC comment、block comment、trailing comma、syntax error 和 schema error。
- launch/attach configuration 精确选择、missing name、duplicate name 和 missing file。
- workspace containment 与 allow-external override。
- timeout、cancellation、DAP terminated-before-stop 与 stopped-with-frame。
- breakpoint/logpoint add/remove/list/clear。
- variable scope、1/50/51 names、child cap、GDB memory cap 和 expression error。
- optional `nvim-dap-ui` installed/missing 两种模式。

## Mock 规则

只 mock 外部边界：Neovim API、`nvim-dap`、network、process、filesystem 和 clock。内部 parser、validator、mapper、state machine 和 tool orchestration 尽量使用真实组合。

mock 不得隐藏实际 schema、event order 或错误转换。跨边界 wiring 必须由 integration test 或真实 Neovim 验证。

## launch.json E2E fixture

`tests/fixtures/rust-codelldb/` 提供最小 Rust debuggee 与 `.vscode/launch.json`，至少包含一个 launch 和一个 attach configuration。完整流程应验证：

- configurationName 精确选择。
- breakpoint 命中。
- step_over、step_into、step_out、continue 和 pause。
- stack/location state 更新。
- list/get variables 与 evaluate expression。
- restart、stop 与资源清理。
- 可选 dap-ui 自动打开/关闭。

fixture 只用于验证本插件，不得依赖用户私人 project 或 machine-specific absolute path。环境缺少 CodeLLDB 时必须报告 blocker，不得改用 fake adapter 并宣称 E2E 通过。

## debugger-mcp 使用规则

本项目自身的标准断点调试路径为：

- Rust sidecar：CodeLLDB。
- Neovim Lua：one-small-step-for-vimkind/OSV。

以下问题必须通过用户开启的 VS Code `debugger-mcp` 调试：

- 难复现或测试无法解释的失败。
- Lua/Rust/Neovim/DAP 跨进程状态错误。
- 并发、async、event ordering、hang、timeout 或 lifecycle race。
- crash、错误变量值、错误 frame/thread 或 adapter request mismatch。

标准流程：复现 → 建立 hypothesis → 在关键入口或 state transition 设置 breakpoint/logpoint → 启动对应 launch configuration → step 与检查具体变量 → 找到共享根因 → 修复 → 加 regression test → 重跑场景 → 清理 breakpoint。

不得把 debugger 只当作最终截图工具，也不得只在异常抛出行停住就宣布根因。

如果 `debugger-mcp` 未连接而本规则要求使用，必须请用户开启 VS Code client 并等待。

## neovim-mcp 体感验收

用户流程变更、release candidate 和 MCP/DAP wiring 完成时，必须通过 `neovim-mcp` 在真实 Neovim 覆盖完整流程：

1. 确认插件默认没有监听 MCP port。
2. 验证缺失、非法和已占用 port 的英文错误。
3. 执行 `:DebuggerMcpStart <port>` 并完成 MCP initialize。
4. 使用 fixture 的 `.vscode/launch.json` 与具名 configuration 调用 `start_debugging`。
5. 验证 dap-ui installed/missing 路径。
6. 添加 breakpoint 与 logpoint，执行 launch/attach、暂停、继续和三种 step。
7. 查看 breakpoint、frame、变量和 expression 结果。
8. restart 与 stop debug session。
9. 执行 `:DebuggerMcpStop`，确认 DAP、UI、HTTP、MCP session、sidecar 和 Lua state 均清理。
10. 验证 sidecar crash 后只报告并允许手动重启，不进入 restart loop。

如果 `neovim-mcp` 未连接，必须请用户开启并等待。headless test 不能替代这套体感验收。

## 失败、flaky 与重试

- 不得以重复运行直到通过来掩盖 failure。
- 首次失败应保存 command、error、log 和相关 runtime state，然后定位根因。
- 只有确认 external/environmental cause 后才可重跑，并记录依据。
- flaky test 必须修复 deterministic control，或明确标为 blocker；不得降低断言、删除测试或加入无依据 sleep。

## CI 基线

首版 CI 必须覆盖 Linux、Neovim stable 和 Rust stable edition 2024。至少执行：

- StyLua check。
- Selene lint。
- LuaLS diagnostics 或项目等价 type-check gate。
- `mini.test` suite。
- `cargo fmt --check`。
- `cargo clippy --all-targets --all-features -- -D warnings`。
- `cargo test --all-targets --all-features`。
- Lua/Rust Msgpack-RPC 和 Streamable HTTP integration tests。

真实 CodeLLDB 与 `neovim-mcp` 体感测试可以是受控 acceptance job 或本地 release gate，但执行证据必须保留，不能从完成标准中删除。

## 长任务证据

长任务的每个 `todo.md` checklist 项在勾选时记录：

- 实际 command 或 MCP tool。
- 覆盖的场景。
- pass/fail 结果。
- 若曾失败，根因与修复后的重跑结果。

最终 `Verification` 汇总所有相关场景、工具、结果和剩余 blocker。
