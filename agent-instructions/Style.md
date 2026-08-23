# Coding Style

## 语言与产品文本

- 代码标识、public API、MCP schema、command output、日志和 error message 只使用英文。
- 本项目不做 i18n，不创建 translation key、locale 文件或 localization helper。
- 规则和设计文档可使用中文正文，但路径、命令、类型、字段和 error code 保持英文。

## 文件与模块规模

- Lua、Rust 和测试源码每个文件不得超过 400 行。
- `Cargo.lock`、机器生成代码和机器生成 artifact 豁免；手写配置和源码不豁免。
- 一个文件只承担一个 coherent responsibility。接近 400 行时应在超限前按 capability 或 layer 拆分。
- 不创建 catch-all `helpers.lua`、`misc.rs` 或 `common` module。跨领域通用 helper 放入对应语言的 `util/`；domain-specific helper 留在所属 module。
- 函数超过约 50 行或嵌套超过 3 层时必须 review 是否拆分，但 50 行是设计提示而非机械硬限制。优先判断职责、数据流和测试 seam。

## Lua

- 模块返回局部 `M` table，禁止新增 global variable。
- runtime state 放入职责明确的 state object，不把可变 singleton state 随意藏在 module scope。
- 依赖显式传入或由 composition root 组装，测试不得依赖加载顺序或隐式全局。
- `require` 默认放在文件顶部。仅以下情况允许局部 `require`，并在代码中简短说明 why：
  - optional dependency，例如 `nvim-dap-ui`。
  - 避免实际 circular dependency。
  - 有意 lazy-load 以保护 startup 或 runtime 边界。
- 所有 named function，不论 public 或 private，都必须有说明、`---@param`、`---@return` 和必要的 `---@async`/error 语义。显而易见的单行 anonymous callback 可豁免。
- 所有 type、class、alias、field 和 public module surface 使用完整 EmmyLua annotation。
- 公共 API 提供可运行或可直接改写的 usage example。
- 使用 StyLua 统一格式，Selene lint，LuaLS 做 EmmyLua type diagnostics。

## Rust

- 使用 stable Rust edition 2024 与标准 `rustfmt`。
- CI 执行 `cargo clippy --all-targets --all-features -- -D warnings`。不整体 deny `clippy::pedantic`；只选择有价值且低噪声的 lint。
- 每个 named type、trait、enum、variant、function 和 method，包括 private item，都必须有适合可见性的 doc comment；public API 说明输入、输出、错误、async/lifecycle 语义，并在适用时提供示例。
- production path 禁止 `unwrap()`、`expect()`、`panic!()` 和 `unreachable!()`。
- production code 禁止 `unsafe`。如果底层 API 确实无法避免，必须先获得用户批准，隔离到最小 module，写 `SAFETY:` 注释、不变量和专门测试。
- error 必须通过 typed result 返回；不得用 panic 处理网络、配置、RPC、DAP 或用户输入失败。

## 命名

- Lua local、function、module 与 Rust function/module/field 使用 `snake_case`。
- Rust type、trait 与 enum 使用 `PascalCase`，constant 使用 `SCREAMING_SNAKE_CASE`。
- MCP tool name 保持 Microsoft DebugMCP 的 `snake_case`。
- 外部 MCP DTO 字段保持上游 camelCase；在 serde 或显式 mapper 边界转换到内部 snake_case。
- 名称描述 domain intent，不使用 `data`、`manager`、`handler` 等模糊词，除非类型确实承担该明确角色。

## 数据流、状态与异步

- 参数显式传递，返回有用结果；禁止远距离 mutation 和 bidirectional hidden coupling。
- side effect 放在 HTTP、Msgpack-RPC、Neovim API、DAP 和 filesystem 边界。
- 核心 parse、validate、select、transition 和 mapping 逻辑保持 pure 或可用简单 fake 调用。
- Neovim main loop 只执行短小状态变化和 API 调用。IO、JSONC parsing、HTTP、timeout 与等待放在 Rust/Tokio sidecar。
- 不 busy-wait，不用 blocking sleep 等待 DAP event。使用 event、notification、channel 和 cancellable timeout。
- single active session state 必须通过明确 state machine 管理，不用多个 boolean 拼出隐式状态。

## 配置

- timeout、bind host、binary path、log level、feature toggle 和行为开关统一进入 validated config。
- raw user config 与 validated immutable runtime snapshot 分离。
- 不硬编码可能跨环境或版本改变的 path、port、timeout 或 adapter name。
- config 修改只影响下一次 Start；运行中的 sidecar 使用启动时 snapshot，禁止半途产生 mixed state。

## 错误

- Lua、Rust 与 MCP 边界使用稳定 error code、英文 message、可操作 context 和可选 cause。
- 内部 error 转换集中处理，不能在每个 call site 手写不一致文本。
- MCP 响应不得直接暴露 Rust backtrace、panic、Neovim 内部 stack trace 或不必要的 filesystem 内容。
- unknown state、invalid transition、timeout、missing dependency、invalid launch config 和 RPC disconnect 使用不同 code。
- 修复 bug 时先定位共享 failure mechanism；不得为单个 reported input 枚举特例。

## 日志

- Rust 使用 `tracing`，Lua 使用项目统一 logger facade。
- log level 至少支持 `DEBUG`、`INFO`、`WARN`、`ERROR`，由 validated config 控制。
- 关键操作必须在 detected/started/succeeded/failed/cancelled/timed_out 等状态点记录 stable event name 与必要 identifier。
- 记录 Start/Stop、sidecar spawn/exit、HTTP bind、MCP session、tool call、RPC request、DAP lifecycle 与 launch selection。
- 不记录完整变量值、表达式结果、source file 内容、环境变量或其他可能敏感的大对象。
- 高频协议细节只进 DEBUG，不使用 `vim.notify` 制造噪声；用户需要行动的 WARN/ERROR 才进入 Neovim notification。

## 注释与文档

- 普通注释只解释 why、protocol constraint、lifecycle invariant、workaround 或不明显的安全原因，不逐行复述代码。
- workaround 必须链接上游 issue、spec 或说明移除条件。
- public API、MCP schema、配置项和架构变化必须同步更新 README 或项目文档；架构变化必须实时更新 `Project.md`。
- 文档示例必须与当前 API 一致，适用时由测试或 smoke check 执行。

## 依赖

- 写 parser、protocol client、retry、schema、queue 或类似通用能力前必须执行 `$dependency-code` 的 DuckDuckGo + GitHub 查找流程。
- 首选成熟、活跃、API 清楚且减少自定义代码的依赖；拒绝 abandoned、过度庞大、许可证不兼容或只节省极少代码的包。
- 当前批准 `rmcp`、`nvim-rs`、`jsonc-parser`。新增或替换核心依赖必须记录候选、维护状态、许可证、选择原因和被拒绝方案。
- Cargo.toml 使用受控 semver range 并提交 Cargo.lock。依赖升级需要 changelog review 和完整测试。
- 默认许可证白名单：MIT、Apache-2.0、BSD、ISC、Zlib。白名单外依赖必须先问用户。唯一已批准例外是用户本地构建场景下的 `nvim-rs 0.9.2`（LGPL-3.0）；预编译分发前必须重新审查合规。

## Testability

- 只在外部边界 mock Neovim API、`nvim-dap`、network、process、filesystem 和 clock；内部 module 尽量真实组合。
- clock、timeout、process launcher、filesystem root 和 transport 通过窄 interface 或 collaborator 注入。
- 每个新 branch 都要有可观察 behavior 证明其正确；测试要求详见 `TestAndDebug.md`。
