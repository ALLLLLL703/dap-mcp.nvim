# dap-mcp.nvim Agent Instructions

本文件是仓库级 agent 入口。开始任何项目任务前，必须按顺序完整读取并遵守以下全部规则文件：

1. [General.md](agent-instructions/General.md) — 项目目的、强制技能与任务治理。
2. [Mcp.md](agent-instructions/Mcp.md) — MCP、代码探索、文档、搜索与调试工具路由。
3. [Project.md](agent-instructions/Project.md) — 当前架构、功能边界与目录责任。
4. [Style.md](agent-instructions/Style.md) — Lua、Rust、API、配置、日志与文档风格。
5. [TestAndDebug.md](agent-instructions/TestAndDebug.md) — 测试层级、debugger 与真实 Neovim 验收。

以上文件共同组成完整规则，不能只读取与当前任务看似最相关的一份。发生冲突时，按以下优先级处理：

1. 用户在当前对话中的明确要求。
2. 本文件与 `agent-instructions/General.md` 的项目级硬约束。
3. 其余规则文件中更具体、与当前任务更接近的规则。
4. 已加载 skill 的通用建议。

如果规则仍然冲突、缺少关键信息或需要改变已经确认的产品边界，必须停止猜测并执行 `$grill-me` 访谈。

新增、重命名或删除任何 `agent-instructions/*.md` 时，必须在同一变更中更新本文件的显式链接、读取顺序和说明，确保本文件始终引用目录下全部 Markdown 文件。
