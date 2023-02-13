use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref DEFAULT_JSON_RPC: String = String::from("2.0");
}

/// Follows: https://ethereum.org/en/developers/docs/apis/json-rpc/#curl-examples
#[derive(Serialize, Deserialize)]
pub struct JSONRPCParam {
    pub id: u16,
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

impl JSONRPCParam {
    #[allow(dead_code)]
    pub fn new(id: u16, method: String, params: serde_json::Value) -> Self {
        JSONRPCParam {
            id,
            jsonrpc: DEFAULT_JSON_RPC.clone(),
            method,
            params,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JSONRPCResponse<T: Serialize> {
    pub id: u16,
    pub jsonrpc: String,
    pub result: T,
}
