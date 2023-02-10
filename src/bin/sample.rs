use ipc_client::{JsonRpcClientImpl, LotusApi, LotusClient};

#[tokio::main]
async fn main() {
    env_logger::init();

    let h = JsonRpcClientImpl::new("".parse().unwrap(), None);
    let n = LotusClient::new(h);
    println!(
        "wallets: {:?}",
        n.wallet_new(ipc_client::WalletKeyType::Secp256k1).await
    );
}
