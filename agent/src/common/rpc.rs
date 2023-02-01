use serde::{Deserialize, Serialize};

/// Follows: https://ethereum.org/en/developers/docs/apis/json-rpc/#curl-examples
#[derive(Serialize, Deserialize)]
pub struct JSONRPCParam {
    pub id: u16,
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct JSONRPCResponse<T: Serialize> {
    pub id: u16,
    pub jsonrpc: String,
    pub result: T,
}
