use ipc_agent::jsonrpc::JsonRpcClientImpl;
use ipc_agent::lotus::LotusJsonRPCClient;

pub fn http_json_rpc(url: &str, bearer_token: Option<&str>) -> JsonRpcClientImpl {
    JsonRpcClientImpl::new(url.parse().unwrap(), bearer_token)
}

pub fn lotus_http_json_rpc_client(url: &str, bearer_token: Option<&str>) -> LotusJsonRPCClient<JsonRpcClientImpl> {
    let json_rpc = http_json_rpc(url, bearer_token);
    LotusJsonRPCClient::new(json_rpc)
}
