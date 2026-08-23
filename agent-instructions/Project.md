# 项目架构与功能

## 文档状态

本文件描述截至 2026-08-23 已实现并通过验收的架构。每次新增、删除、移动模块，改变公共 API、MCP schema、进程协议、状态机、依赖或测试拓扑时，必须在同一任务中实时更新本文件，并同步检查 `AGENTS.md`、`Style.md` 和 `TestAndDebug.md` 是否仍一致。

## 系统边界

```text
AI agent
  -> MCP Streamable HTTP on loopback /mcp
Rust sidecar
  -> Neovim native Msgpack-RPC over parent stdin/stdout
Lua plugin in the user's current Neovim
  -> nvim-dap
Debug adapter
  -> debuggee
```

Rust sidecar 负责 MCP transport、tool schema、HTTP 安全、请求超时、JSONC 解析和跨边界错误映射。Lua plugin 负责 Neovim 命令、sidecar 生命周期、`nvim-dap` 调用、DAP event 监听和可选 `nvim-dap-ui` 协调。不得让 Rust 直接实现一个新的 DAP client，也不得让 Lua 承担 HTTP server。

## 运行时生命周期

### 启动

- 插件默认关闭，不自动启动 MCP server。
- 用户调用 `:DebuggerMcpStart <port>`。
- `<port>` 必填，只接受未占用的 `1..65535` TCP port。
- Lua 验证配置、`nvim-dap` 和 sidecar binary，然后以 `rpc=true` 拉起本地 Rust binary。
- Rust 使用 `nvim-rs::create::new_parent()` 连接父 Neovim，并只在 loopback 地址启动 Streamable HTTP `/mcp` endpoint。
- runtime state 至少区分 `stopped`、`starting`、`running`、`stopping`、`failed`，非法 state transition 返回结构化错误。

### 停止

`:DebuggerMcpStop` 必须按顺序：

1. 终止活动 DAP session。
2. 如果 `nvim-dap-ui` 已安装并由本插件打开，则关闭 UI。
3. 结束 MCP sessions 并停止接受新请求。
4. 关闭 HTTP listener。
5. 优雅退出 Rust sidecar，清理 Lua runtime state。

Neovim 退出时执行同等清理。sidecar 异常退出时记录 exit code 与 stderr、清理状态并报告英文错误；禁止自动重启，用户必须重新执行 Start。

## 配置 API

公共入口为：

```lua
require("dap-mcp").setup(opts)
```

配置至少覆盖 sidecar binary path、loopback bind host、180 秒 operation timeout、log level、是否允许工作区外文件和 `nvim-dap-ui` 自动打开。配置必须先验证，再形成不可变 runtime snapshot；测试可注入短 timeout。Start 命令的 port 是必填会话参数，不由默认配置替代。

首版只允许 `localhost`、`127.0.0.1` 和 `::1`。虽然 bind host 可配置，但不得监听 LAN address 或 `0.0.0.0`；远程使用通过 SSH tunnel 完成。

## MCP tool surface

首版保持 Microsoft DebugMCP 核心 tool name：

- `start_debugging`
- `stop_debugging`
- `step_over`
- `step_into`
- `step_out`
- `continue_execution`
- `pause_execution`
- `restart_debugging`
- `add_breakpoint`
- `add_logpoint`
- `remove_breakpoint`
- `clear_all_breakpoints`
- `list_breakpoints`
- `list_variable_names`
- `get_variables_values`
- `evaluate_expression`

外部 schema 保持上游 camelCase 字段，例如 `fileFullPath`、`workingDirectory`、`configurationName` 和 `variableNames`；Rust/Lua 内部使用各自生态命名，并在 DTO 边界显式映射。

首版只有一个活动 DAP debug session。启动新 session 时如果旧 session 仍活动，返回稳定错误，而不是隐式替换。

## start_debugging 与 launch.json

`start_debugging` 必须接收 `fileFullPath`、`workingDirectory` 和 `configurationName`：

- 每次调用都重新读取 `<workingDirectory>/.vscode/launch.json`。
- 使用 `jsonc-parser` 支持 line comment、block comment 和 trailing comma。
- 只接受具名且唯一匹配的 `request: "launch"` 或 `request: "attach"` configuration。
- 找不到文件、配置缺失、名称重复或 schema 无效时返回稳定 error code、候选 configuration name 和英文修复提示。
- 默认拒绝 `fileFullPath` 位于 `workingDirectory` 外；只有 setup 显式允许时才能放行。
- 不生成猜测的默认 adapter configuration，不打开 Neovim 配置选择器。
- 解析后的 configuration 交给 `nvim-dap` 的 `dap.run(config)`；adapter 注册仍由用户的 Neovim 配置负责。

保留上游 optional `testName` 字段以兼容调用方，但首版收到非空 `testName` 时必须返回 `UNSUPPORTED_TEST_TARGET`。Neovim 没有跨语言等价的 VS Code Testing API，不得伪造通用支持。

首版不承诺 VS Code 的 compounds、pre/post tasks、inputs 或 command-variable 完整兼容；新增这些能力前必须更新本文件、schema 和测试矩阵。

## DAP 与 UI

`nvim-dap` 是必需外部依赖，缺失时 Start 失败并给出英文安装提示。具体 debug adapter 由用户配置，插件不下载或静默注册 adapter。

`nvim-dap-ui` 是可选增强：存在时 `start_debugging` 自动打开；不存在时继续调试并记录低噪声提示。不得把 UI 状态作为 debug session 状态来源。

Lua 必须监听 `nvim-dap` lifecycle event，并通过 `vim.rpcnotify()` 向 Rust sidecar 发送停止、继续、终止、输出和 session 状态更新。Rust tool handler 等待明确事件或 timeout，不在 Neovim main loop 中 busy-wait。

## HTTP 与变量安全边界

- Streamable HTTP 使用 stateful MCP session，支持初始化、复用、关闭和 graceful shutdown。
- 每个请求校验 Host、Origin 和 port，阻止 DNS rebinding 与错误端口访问。
- 仅 loopback bind；不实现远程 unauthenticated mode。
- 默认 operation timeout 为 180 秒，tool backstop 为 210 秒。
- `get_variables_values` 接受 1 到 50 个显式变量名，不支持 wildcard。
- 复杂值的后代展开遵循上游 100 fields 总上限；GDB memory read 遵循 4096 bytes 上限。
- `evaluate_expression` 保留完整表达式能力，因此 endpoint 只能运行在可信本机边界。
- 按项目决定，不对变量或表达式结果做 automatic secret redaction；文档和日志不得宣称存在脱敏保护。

## 当前目录分配

以下目录与模块已经实现：

```text
plugin/
  dap-mcp.lua                 Neovim command registration only
lua/dap-mcp/
  init.lua                    public setup API
  config.lua                  defaults, validation, runtime snapshot
  commands.lua                Start/Stop command adapters
  lifecycle.lua               sidecar process and runtime state machine
  dap_client.lua              narrow nvim-dap operations
  dap_requests.lua            callback-based scopes, variables and evaluate
  dap_events.lua              DAP listener to rpc notification bridge
  rpc.lua                     request-id response envelope and dispatch
  launch.lua                  launch configuration handoff to dap.run
  ui.lua                      optional nvim-dap-ui integration
  logger.lua                  Lua logging facade
  errors.lua                  stable Lua-side error codes
  util/                       cross-cutting Lua helpers only
sidecar/
  Cargo.toml
  Cargo.lock
  src/
    main.rs                   composition root and graceful shutdown
    config.rs                 validated process config
    error.rs                  stable errors and boundary mapping
    logging.rs                tracing setup
    launch.rs                 JSONC parse, validate, select and workspace expansion
    security.rs               Host and Origin validation
    state.rs                  single debug-session state
    mcp/
      http.rs                 stateful Streamable HTTP and request guard
      operations.rs           timeout-aware debugger operations
      schema.rs               external camelCase and internal DTOs
      server.rs               16 rmcp tool handlers
    nvim/
      bridge.rs               typed parent-Neovim request/response bridge
      handler.rs              response, DAP event and shutdown notifications
tests/
  lua/                        mini.test unit and Neovim integration tests
  fixtures/
    rust-codelldb/             real launch.json E2E debuggee
  integration/
    codelldb_init.lua          real Neovim/CodeLLDB acceptance bootstrap
docs/
  history/                    archived completed long-task checklists
agent-instructions/           project agent rules
style.md                      current implementation plan
todo.md                       active long-task checklist and evidence
```

不要为了凑目录创建空 module。只有职责实际出现时才创建最小 coherent module，同时更新本文件的 implemented 状态。

## 构建与交付

首版不下载预编译 binary。用户必须在 `sidecar/` 使用 stable Rust edition 2024 本地 `cargo build --release`，并在 setup 中指向生成的 binary；后续如增加 release binary 下载，必须先执行 `$grill-me` 并加入 checksum、平台矩阵与供应链验证。

Cargo.toml 使用受控 semver compatibility range，并提交 Cargo.lock。升级依赖必须查 changelog、通过完整验证。默认只接受 MIT、Apache-2.0、BSD、ISC 和 Zlib 许可证；其他许可证必须先询问用户。

`nvim-rs 0.9.2` 的 LGPL-3.0 已由用户作为明确例外批准，仅用于用户本地构建的首版 sidecar。未来发布预编译 binary 前必须重新完成 LGPL linking/distribution 合规审查，不得把该例外扩展到其他依赖。
