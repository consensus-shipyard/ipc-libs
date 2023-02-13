use std::time::Duration;
use url::Url;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl, NO_PARAMS};

struct CheckpointContext {
    subnet: String,
    jsonrpc_endpoint: Url,
    auth_token: String,
    account: String,
    period: u32,
}


#[tokio::test]
#[ignore]
async fn monitor_subnet() {
    let url = Url::parse("https://api.node.glif.io/rpc/v0").unwrap();
    let period: u64 = 10;

    let client = JsonRpcClientImpl::new(url, None);

    for _ in 1..=10 {
        let response = client.request("Filecoin.ChainHead", NO_PARAMS).await.unwrap();
        let result = response.get("result").unwrap();
        let blocks = result.get("Blocks").unwrap();
        let height = result.get("Height").unwrap().as_u64().unwrap();
        println!("{}", blocks);
        println!("{}", height);

        if height % period == 0 {
            println!("Height {} is checkpoint tipset", height);
        }

        std::thread::sleep(Duration::from_secs(30));
    }
}