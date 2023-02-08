use fil_actors_runtime::cbor;
use ipc_client::{InstallActorParams, JsonRpcClientImpl, LotusApi, MpoolPushMessage};

#[tokio::main]
async fn main() {
    env_logger::init();

    let h = JsonRpcClientImpl::new("http://localhost:1234/rpc/v0".parse().unwrap(), Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.dN5qHZ8dzXScslA1185ADefb-bGTAJVejivU_tiiqt0"));
    let n = LotusApi::new(h);
    let r = n
        .mpool_push_message(MpoolPushMessage::new(
            fil_actors_runtime::INIT_ACTOR_ADDR,
            None,
            3,
            cbor::serialize(&InstallActorParams { code: vec![] }, "")
                .unwrap()
                .to_vec(),
        ))
        .await
        .unwrap();
    println!("{r:?}");

    let m = n
        .state_wait_msg(r.get_root_cid().unwrap(), r.nonce)
        .await
        .unwrap();
    println!("state wait: {m:?}");
}
