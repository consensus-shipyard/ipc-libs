use serde::{Serialize, Deserialize};
use crate::server::DEFAULT_JSON_RPC_SERVER_VERSION;

/// List of error codes for json rpc, see more: https://www.jsonrpc.org/specification#error_object
const INVALID_REQUEST_CODE: i32 = -32600;

/// The json rpc result response. It is the standard form our json-rpc and follows
/// the spec: https://www.jsonrpc.org/specification#response_object
#[derive(Debug, Serialize, Deserialize)]
pub struct JSONRPCResultResponse<T> {
    pub id: u16,
    pub jsonrpc: String,
    pub result: T,
}

impl <T: Serialize> JSONRPCResultResponse<T> {
    pub fn new(id: u16, result: T) -> Self {
        Self { id, jsonrpc: String::from(DEFAULT_JSON_RPC_SERVER_VERSION), result }
    }
}

/// The json rpc error response error object. It is following the json rpc spec: https://www.jsonrpc.org/specification#error_object
#[derive(Debug, Serialize, Deserialize)]
pub struct JSONRPCError<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>
}

/// The json rpc error response. It is the standard form our json-rpc and follows the spec: https://www.jsonrpc.org/specification#response_object
#[derive(Debug, Serialize, Deserialize)]
pub struct JSONRPCErrorResponse<T> {
    pub id: u16,
    pub jsonrpc: String,
    pub error: JSONRPCError<T>,
}

impl JSONRPCErrorResponse<()> {
    pub fn invalid_request(id: u16) -> Self {
        Self {
            id,
            jsonrpc: String::from(DEFAULT_JSON_RPC_SERVER_VERSION),
            error: JSONRPCError {
                code: INVALID_REQUEST_CODE,
                message: String::from("Invalid Request"),
                data: None
            }
        }
    }
}
impl <T: Serialize> JSONRPCErrorResponse<T> {
    pub fn new(id: u16, error: JSONRPCError<T>) -> Self {
        Self { id, jsonrpc: String::from(DEFAULT_JSON_RPC_SERVER_VERSION), error }
    }
}
