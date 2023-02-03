use base64::Engine;
use cid::Cid;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use ipc_client::{ExecParams, JsonRpcClientImpl, LotusApi, MpoolPushMessage};
use std::str::FromStr;

#[tokio::main]
async fn main() {
    env_logger::init();

    let code_cid =
        Cid::from_str("bafk2bzaceaodptkf3t7ki47wr5cmxu7jdkfshabohkemjitq2lolrac64h4s4").unwrap();
    let constructor_params = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode("gmUvcm9vdBQ=")
        .unwrap();

    let h = JsonRpcClientImpl::new("http://localhost:1234/rpc/v0".parse().unwrap(), Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.dN5qHZ8dzXScslA1185ADefb-bGTAJVejivU_tiiqt0"));
    let n = LotusApi::new(h);
    let r = n.mpool_push_message(MpoolPushMessage::new(
        fil_actors_runtime::INIT_ACTOR_ADDR,
        Address::from_str("f3vzsgqv4dkugomlsmajsio63365tto7g2srp5for5v57mgwx2mnkogaigshnkold74bse75fwsbu4f4kacsqa").unwrap(),
        2,
        cbor::serialize(&ExecParams { code_cid, constructor_params }, "").unwrap().to_vec(),
    ))
    .await
    .unwrap();

    let m = n
        .state_wait_msg(r.get_root_cid().unwrap(), r.nonce)
        .await
        .unwrap();
    println!("state wait: {m:?}");
}
