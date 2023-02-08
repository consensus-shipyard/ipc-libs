use ipc_client::{JsonRpcClientImpl, LotusApi};

#[tokio::main]
async fn main() {
    env_logger::init();

    let h = JsonRpcClientImpl::new("".parse().unwrap(), None);
    let n = LotusApi::new(h);
    println!(
        "wallets: {:?}",
        n.wallet_new(ipc_client::WalletKeyType::Secp256k1).await
    );
}
