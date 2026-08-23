//! Neovim Msgpack-RPC bridge and event handling.

mod bridge;
mod handler;

pub use bridge::NvimBridge;
pub use handler::{BridgeShared, DebugEvent, NvimHandler};
