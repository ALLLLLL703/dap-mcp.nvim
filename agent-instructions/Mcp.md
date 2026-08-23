# MCP 与工具路由

## 总原则

工具调用必须选择能提供最高语义信号的 MCP。首选工具不可用、调用失败或覆盖不足时，只做一次聚焦尝试，然后立即使用规定 fallback，并在结果中说明实际证据来自哪里。不得对失败 MCP 反复重试。

本项目不使用 `codebase-memory-mcp` 进行代码探索。

## 上游 GitHub 源码与对象

查看 Microsoft DebugMCP、`nvim-dap`、`nvim-dap-ui`、依赖库或其他上游仓库的源文件、issue、PR、release、commit 和 metadata 时：

1. 首选 GitHub MCP。
2. GitHub MCP 不可用或覆盖不足时，回退到 `gh` CLI。
3. 仍不可用时，才使用 Open WebSearch 或目标网页读取，并说明 fallback 原因。

不得仅凭记忆描述上游当前行为。借鉴上游实现时，记录所查看的 commit、tag 或文件路径；只借用适合本项目的最小结构，不整段复制架构。

## Library、API、SDK 与 CLI 文档

选择或使用 library、API、SDK、framework 或 CLI 时必须使用 Context7：

1. 除非用户给出精确 `/org/project` ID，否则先调用 `resolve-library-id`。
2. 按名称匹配、文档相关性、source reputation、snippet 数量和 benchmark 选择最佳 ID。
3. 每个独立概念使用一次聚焦的 `query-docs`。
4. Context7 不可用或没有覆盖时，回退 Open WebSearch，再回退官方文档或 GitHub 源码。

新增通用能力前还必须遵守 `$dependency-code`：先用 DuckDuckGo 和 GitHub MCP 比较维护状态、API、许可证与生态采用，再决定是否使用依赖。当前批准的核心 Rust 依赖为 `rmcp`、`nvim-rs` 和 `jsonc-parser`；替换它们属于架构决策，必须先执行 `$grill-me`。

## 本地代码探索

仓库根存在 `.codegraph/` 时，理解架构、定位 symbol、调用链或改动影响前必须先使用：

```text
codegraph explore "<symbols or focused question>"
```

如果可用等价 CodeGraph MCP，可优先使用 MCP；本项目规则至少保证 CLI 路径可用。不得自行初始化或重建 `.codegraph/`，索引由用户决定。

仅在以下情况回退原始搜索和文件读取：

- 仓库没有 `.codegraph/`。
- CodeGraph 返回不足、过期或失败。
- 搜索 string literal、error message、配置值或非代码文件。

fallback 时优先使用 `rg` 与 `rg --files`，并说明 CodeGraph 未使用或不足的原因。

## Semantic code operation

以下操作必须首选已配置的 LSP MCP：

- rename symbol。
- definition、declaration、implementation。
- references 与 call hierarchy。
- document/workspace symbols。
- diagnostics。

操作前初始化正确 workspace 和 document。rename 必须先查看完整 workspace edit，再应用，并运行项目的 formatter、lint、type check 和相关测试。LSP MCP 不可用、language server 不支持或结果不足时，才可回退本地编辑与搜索；必须记录 fallback 原因。

LSP diagnostics 只是 editor-state 证据，不能替代正式测试、compiler 或 linter。

## 公共 Web 搜索

搜索具体事项、当前信息、错误文本、实现比较或非结构化网页时：

1. 首选内置 WebSearch 或 Open WebSearch。
2. 使用 Open WebSearch 时必须显式指定 DuckDuckGo engine。
3. 不使用 Bing。
4. 技术问题优先引用官方文档、标准、研究论文或上游源码。

## Debugger MCP

调试复杂失败时必须使用 `debugger-mcp`，由用户提供已开启的 VS Code client。适用场景包括：

- 难以稳定复现的错误。
- Lua/Rust/Neovim/DAP 跨进程状态问题。
- 并发、异步、生命周期、hang 或 timeout。
- 测试和日志无法解释的错误状态。

先加载 debugger 对应的调试 workflow skill，再设置聚焦 breakpoint 或 logpoint、运行、逐步检查变量与状态，并追到根因。不得只看最终异常位置。调试完成后清理 breakpoint。

`debugger-mcp` 未连接但当前验证规则要求使用它时，必须请用户开启并等待；不得用日志或 CLI 结果冒充断点调试证据。

## Neovim MCP

所有本地 Neovim 集成测试、命令交互、UI 行为与最终体感验收必须使用 `neovim-mcp` 驱动真实 Neovim 实例。覆盖范围见 `TestAndDebug.md`。

`neovim-mcp` 未连接但当前任务需要强制体感验收时，必须请用户开启并等待；headless Neovim 只能作为补充自动化证据，不能替代规定的真实交互验收。

## 本地命令

本地文件读取、build、test、format、lint、Git working tree 操作与已确认的项目脚本直接使用 shell。不得为了形式上的 MCP-first 把正常本地构建转成远程工具调用。
