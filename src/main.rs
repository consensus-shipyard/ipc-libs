use crate::jsonrpc::JsonRpcClientImpl;

mod jsonrpc;

fn main() {
    JsonRpcClientImpl::new("".parse().unwrap(), None);
}
