use crate::jsonrpc::JsonRpcClientImpl;

mod jsonrpc;

fn main() {
    // just some dummy code to silent unused code warning, change this in actual build
    JsonRpcClientImpl::new("".parse().unwrap(), None);
}
