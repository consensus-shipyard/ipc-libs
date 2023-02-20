//! The ipc-agent json rpc node.

pub mod config;
pub mod jsonrpc;
mod request;
mod response;

const DEFAULT_JSON_RPC_SERVER_VERSION: &str = "2.0";
pub const DEFAULT_JSON_RPC_SERVER_ENDPOINT: &str = "json_rpc";
pub const DEFAULT_JSON_RPC_SERVER_PROTOCOL: &str = "http";
pub const DEFAULT_JSON_RPC_SERVER_ADDR: &str = "127.0.0.1:3030";
