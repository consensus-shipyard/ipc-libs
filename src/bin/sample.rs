use ipc_client::{JsonRpcClientImpl, LotusApi};

#[tokio::main]
async fn main() {
    env_logger::init();

    let h = JsonRpcClientImpl::new("".parse().unwrap(), None);
    let n = LotusApi::new(h);
    println!("wallets: {:?}", n.wallet_list().await);
}
