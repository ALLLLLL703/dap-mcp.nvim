# 发布与 dap-mcp skill 清单

- [x] 阶段 1：加入 atomic commit 规则，完成发布前卫生检查，并提交当前插件基线。
  - 规则：`agent-instructions/General.md` 已要求每个独立阶段、小功能、行为改动或 bug fix 验证后立即 atomic commit。
  - 发布卫生：`.codex/` 已忽略，避免提交本机 LSP workspace、MCP port 和 Neovim socket；本机路径与 credential pattern 扫描无命中。
  - 验证：`git diff --check`、StyLua、rustfmt 均通过；mini.test 8 groups / 27 cases 全部通过。
- [x] 阶段 2：在 GitHub 创建并推送公开 `dap-mcp.nvim` 仓库，验证远程默认分支和 README 可访问。
  - 发布：GitHub MCP 创建公开 `ALLLLLL703/dap-mcp.nvim`；`git push -u origin main` 成功。
  - 验证：GitHub MCP 搜索确认 visibility 为 public、default branch 为 `main`、license 为 Apache-2.0，并成功读取远程 `README.md`。
- [ ] 阶段 3：创建并全局安装 dap-mcp 专用 debugging skill，验证 metadata、协议约束与自动发现所需目录结构。

## Verification

- 待完成。
