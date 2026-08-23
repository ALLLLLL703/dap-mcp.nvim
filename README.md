# dap-mcp.nvim

`dap-mcp.nvim` exposes the current Neovim `nvim-dap` client as a local MCP debugger. The server is disabled by default and starts only when you run `:DebuggerMcpStart <port>`.

The plugin reads a named configuration from `.vscode/launch.json`, including JSONC comments and trailing commas, then passes that configuration to `nvim-dap`. Debug adapter installation and registration remain under your Neovim configuration.

## Requirements

- Neovim 0.12 or newer
- `nvim-dap`
- Stable Rust 1.88 or newer for the local sidecar build
- A registered `nvim-dap` adapter referenced by the selected `launch.json` configuration
- Optional: `nvim-dap-ui`

## Build

```sh
cargo build --release --manifest-path sidecar/Cargo.toml
```

The first release intentionally uses a local build. It does not download or install sidecar binaries or debug adapters.

## Setup

```lua
require("dap-mcp").setup({
  binary_path = "/absolute/path/to/dap-mcp.nvim/sidecar/target/release/dap-mcp-sidecar",
  bind_host = "127.0.0.1",
  timeout_seconds = 180,
  log_level = "info",
  allow_external_files = false,
  auto_open_dap_ui = true,
})
```

Register adapters normally through `nvim-dap`. For example, a local CodeLLDB executable can be registered as:

```lua
local dap = require("dap")

dap.adapters.codelldb = {
  type = "executable",
  command = vim.fn.expand("~/.local/share/nvim/mason/bin/codelldb"),
  name = "codelldb",
}
```

## Start and stop

```vim
:DebuggerMcpStart 39001
:DebuggerMcpStop
```

Connect an MCP client to `http://127.0.0.1:39001/mcp`. The port is required on every Start command. Starting the plugin does not start a debug session; call the MCP `start_debugging` tool with:

```json
{
  "fileFullPath": "/absolute/workspace/src/main.rs",
  "workingDirectory": "/absolute/workspace",
  "configurationName": "Launch application"
}
```

`workingDirectory/.vscode/launch.json` is re-read for every call. `configurationName` must match exactly and uniquely. Both `request: "launch"` and `request: "attach"` are accepted. `${workspaceFolder}` is expanded to `workingDirectory`.

## MCP tools

The first release provides these DebugMCP-compatible names:

- Session control: `start_debugging`, `stop_debugging`, `restart_debugging`
- Execution: `step_over`, `step_into`, `step_out`, `continue_execution`, `pause_execution`
- Breakpoints: `add_breakpoint`, `add_logpoint`, `remove_breakpoint`, `clear_all_breakpoints`, `list_breakpoints`
- Inspection: `list_variable_names`, `get_variables_values`, `evaluate_expression`

Only one debug session can be active. Variable reads require a stopped frame. `get_variables_values` accepts 1 to 50 exact names and returns at most 100 descendant names and types for complex values.

## Security

The MCP endpoint is unauthenticated and can evaluate arbitrary target-language expressions. It binds only to `localhost`, `127.0.0.1`, or `::1`, validates Host, Origin, and port headers, and must remain on a trusted local machine. Use an SSH tunnel for remote access; do not expose it to a LAN.

Files outside `workingDirectory` are rejected unless `allow_external_files = true` is explicitly configured. Debug results are not automatically redacted.

## launch.json compatibility

The plugin preserves adapter-specific configuration fields and supports JSONC, a named launch or attach configuration, and `${workspaceFolder}`. It does not emulate VS Code compounds, tasks, inputs, command variables, or the Testing API. A non-empty `testName` returns `UNSUPPORTED_TEST_TARGET`.

## Validation

```sh
stylua --check lua plugin tests scripts
selene lua plugin tests scripts
lua-language-server --check=lua --checklevel=Warning --configpath=.luarc.json
cargo fmt --check --manifest-path sidecar/Cargo.toml
cargo clippy --manifest-path sidecar/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path sidecar/Cargo.toml
```

The real acceptance fixture is under `tests/fixtures/rust-codelldb/` with its connected-Neovim bootstrap at `tests/integration/codelldb_init.lua`.
