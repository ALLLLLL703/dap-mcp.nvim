# 发布与 dap-mcp skill 清单

- [x] 阶段 1：加入 atomic commit 规则，完成发布前卫生检查，并提交当前插件基线。
  - 规则：`agent-instructions/General.md` 已要求每个独立阶段、小功能、行为改动或 bug fix 验证后立即 atomic commit。
  - 发布卫生：`.codex/` 已忽略，避免提交本机 LSP workspace、MCP port 和 Neovim socket；本机路径与 credential pattern 扫描无命中。
  - 验证：`git diff --check`、StyLua、rustfmt 均通过；mini.test 8 groups / 27 cases 全部通过。
- [x] 阶段 2：在 GitHub 创建并推送公开 `dap-mcp.nvim` 仓库，验证远程默认分支和 README 可访问。
  - 发布：GitHub MCP 创建公开 `ALLLLLL703/dap-mcp.nvim`；`git push -u origin main` 成功。
  - 验证：GitHub MCP 搜索确认 visibility 为 public、default branch 为 `main`、license 为 Apache-2.0，并成功读取远程 `README.md`。
- [x] 阶段 3：创建并全局安装 dap-mcp 专用 debugging skill，验证 metadata、协议约束与自动发现所需目录结构。
  - 安装：使用 skill-creator `init_skill.py` 创建 `/home/sanae/.agents/skills/dap-live`，包含 `SKILL.md` 与 `agents/openai.yaml`。
  - 协议：明确使用 `mcp__dap__*`、具名 `.vscode/launch.json` configuration、unsupported `testName`、trusted-local expression 与无自动脱敏边界。
  - 验证：`quick_validate.py` 返回 `Skill is valid!`；无模板 TODO；本会话真实 dap-mcp breakpoint/start/inspect/stop/clear 流程通过。

## Verification

- Git：创建 `ecaf8e6` 初始实现、`14a501a` Apache-2.0 与 `680355e` GitHub 发布证据等 atomic commits；每阶段验证后提交。
- GitHub：公开仓库 `https://github.com/ALLLLLL703/dap-mcp.nvim`，默认分支 `main`，README 与 Apache-2.0 可识别。
- 插件：发布前 `git diff --check`、StyLua、rustfmt、mini.test 8 groups / 27 cases 全部通过；历史全量门禁证据已归档。
- Skill：全局 `$dap-live` 结构和 metadata 校验通过，内容与当前 dap-mcp schema、安全边界及真实运行行为一致。
- 剩余 blocker：无。
