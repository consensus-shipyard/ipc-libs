use fvm_shared::address::Address;
use ipc_client::{JsonRpcClientImpl, LotusApi, MpoolPushMessage};

#[tokio::main]
async fn main() {
    let h = JsonRpcClientImpl::new("".parse().unwrap(), None);
    let n = LotusApi::new(h);
    n.mpool_push_message(MpoolPushMessage::new(
        Address::new_id(0),
        Address::new_id(0),
        0,
        vec![],
    ))
    .await
    .unwrap();
}
