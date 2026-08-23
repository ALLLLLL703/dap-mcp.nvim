//! MCP schemas, tool handlers, and Streamable HTTP transport.

mod http;
mod operations;
mod schema;
mod server;

pub use operations::DebugRuntime;
pub use server::DebugMcpServer;

pub use http::serve_http;
