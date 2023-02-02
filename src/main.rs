mod jsonrpc;

use crate::jsonrpc::{JsonRpcClientImpl, MpoolPushMessage};
use fvm_shared::address::Address;
use jsonrpc::NodeApi;

#[tokio::main]
async fn main() {
    let h = JsonRpcClientImpl::new("".parse().unwrap(), None);
    let n = NodeApi::new(h);
    n.mpool_push_message(MpoolPushMessage::zero_value(
        Address::new_id(0),
        Address::new_id(0),
        0,
        vec![],
    ))
    .await
    .unwrap();
}
