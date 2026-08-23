# General

## 项目目的

`dap-mcp.nvim` 的目标是参考 [Microsoft DebugMCP](https://github.com/microsoft/DebugMCP)，实现一个以 Neovim 和 `nvim-dap` 为 DAP 客户端的本地 debugger MCP server。AI agent 通过 MCP 操作断点、启动或停止 debug session、步进、暂停、继续、查看变量和求值；真正的 DAP 会话由用户当前的 Neovim 实例执行。

插件默认关闭，绝不能在 Neovim 启动时自动暴露 debugger MCP endpoint。用户必须显式执行 `:DebuggerMcpStart <port>` 才能让当前 Neovim 会话成为 MCP 控制的 DAP 客户端；端口必填且严格校验。执行 `:DebuggerMcpStop`、Neovim 退出或 sidecar 异常退出后，必须清理本会话状态和资源。

项目必须支持从 `<workingDirectory>/.vscode/launch.json` 读取具名 `launch` 或 `attach` configuration，并通过 `nvim-dap` 启动。不得把猜测生成的 adapter 配置伪装成用户的 launch configuration。

## 强制技能

开始任何项目任务时，必须加载并遵循以下技能：

- `$maintainable-code`
- `$mcp-first-tool-routing`
- `$grill-me`，适用范围见下文

加载 `$maintainable-code` 时，必须同时加载它的全部强制依赖：

- `$configable-i18-programming-architecture-style`
- `$testable-code`
- `$dependency-code`

缺少任一强制技能或强制依赖时，不得编写代码、review 代码或产出实施计划；先报告缺失并解决加载问题。

### 项目对 configurable/i18n skill 的覆盖

本项目明确不实现 i18n。忽略 `$configable-i18-programming-architecture-style` 中所有 localization、translation key、`LangHelper` 和多语言资源要求。所有产品可见文本、日志、命令反馈、MCP tool description 与错误消息统一使用英文。

该 skill 的其余要求仍然有效，包括配置化、职责拆分、异步或后台执行、关键操作日志、可复用 API 文档和集中管理通用 helper。

### Grill 触发条件

下列情况必须执行 `$grill-me` 的结构化访谈，不能静默选择：

- 新功能、架构或公共 API 设计。
- 重大重构、依赖替换或跨 Lua/Rust 协议变化。
- 测试策略、兼容性、安全边界或用户体验存在取舍。
- 用户需求不明确，且不同解释会产生实质不同结果。

明确的小修、拼写修正或已由现有规则唯一确定的改动不必重复访谈。

## 实施前计划

首次为一个实现任务加载 `$maintainable-code` 时，必须先在根 `style.md` 写入具体实施方案、目标模块、责任边界、测试方法、约束和采用的外部参考，再开始产品代码修改。

根 `style.md` 是当前实施方案，不是长期 coding-style 规则；长期规则位于 `agent-instructions/Style.md`。只有实施方向发生实质变化时才更新根 `style.md`。

## 长任务 checklist

满足以下任一条件即为长任务：

- 预计耗时超过 30 分钟。
- 跨越至少 3 个文件或模块。
- 包含至少 3 个实施阶段。
- 需要多轮测试或调试。

长任务开始前必须在仓库根创建或更新 `todo.md`，使用 `- [ ]` checklist 列出可验证步骤。完成一项后必须立即将对应项更新为 `- [x]`，并在该项下记录执行的命令或工具、覆盖的场景和结果。禁止到任务结束时一次性补勾所有项目。

全部完成后，在 `todo.md` 的 `Verification` 区汇总最终验证。开始下一个长任务前，把已完成的旧任务记录移到 `docs/history/`，再为新任务重写根 `todo.md`；不得丢弃历史证据。

## Git 提交粒度

- 每完成一个可独立验证的实施阶段、一个小功能新增、一个行为改动或一个 bug fix，必须立即创建一次 Git commit。
- 每个 commit 只包含一个 coherent change；不得把互不相关的功能、修复、文档或重构堆进同一个 commit。
- commit 前必须运行与该 change 直接相关的 formatter、lint、测试或真实场景验证；未通过验证的阶段不得标记完成或提交。
- 长任务的 `todo.md` checklist 项与 commit 保持对应：完成并验证一项后先勾选并记录证据，再提交该项涉及的文件。
- 不为满足数量机械拆分无法独立工作的中间状态。若一个最小功能必须跨多个文件才能运行，将这些文件作为同一个 atomic commit。
- 除非用户明确要求，不 amend、squash 或重写已经完成阶段的 commit；后续修正使用新的聚焦 commit。

## 工作原则

- 优先修复根因，不为单个失败样例堆叠硬编码特例。
- 保持变更小而完整，不触碰无关文件或用户已有改动。
- 副作用放在边缘，核心决策保持可测试和确定性。
- 新依赖、新公共 API、新配置项和新目录必须有明确责任与验证方式。
- 所有完成声明必须满足 `agent-instructions/TestAndDebug.md` 的验证门禁。
