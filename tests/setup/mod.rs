use ipc_client::jsonrpc::JsonRpcClientImpl;
use ipc_client::lotus::LotusJsonRPCClient;
use ipc_client::manager::LotusSubnetManager;

pub const MAINNET_JSON_RPC_HTTP_URL: &str = "https://api.node.glif.io/rpc/v0";
pub const LOCAL_JSON_RPC_HTTP_URL: &str = "https://api.node.glif.io/rpc/v0";

pub fn http_json_rpc(url: &str, bearer_token: Option<&str>) -> JsonRpcClientImpl {
    JsonRpcClientImpl::new(url.parse().unwrap(), bearer_token)
}

pub fn lotus_http_json_rpc_client(url: &str, bearer_token: Option<&str>) -> LotusJsonRPCClient<JsonRpcClientImpl> {
    let json_rpc = http_json_rpc(url, bearer_token);
    LotusJsonRPCClient::new(json_rpc)
}
