//! The ipc-agent json rpc node.

pub mod jsonrpc;
pub mod request;
pub mod response;

const DEFAULT_JSON_RPC_SERVER_VERSION: &str = "2.0";
const DEFAULT_JSON_RPC_SERVER_ENDPOINT: &str = "json_rpc";
