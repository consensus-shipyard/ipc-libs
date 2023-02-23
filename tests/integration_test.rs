use ipc_subnet_actor::{ConsensusType, ConstructParams};
use ipc_client::lotus::LotusClient;
use ipc_client::manager::{LotusSubnetManager, SubnetManager};
use crate::setup::{LOCAL_JSON_RPC_HTTP_URL, lotus_http_json_rpc_client};

mod setup;

#[test]
#[ignore] // ignore this test for now as it's still developing
fn test_create_subnet_actor() {
    let bearer_token = std::env::var("IPC_JSON_RPC_TEST_BEARER_TOKEN").ok();
    let lotus_client = setup::lotus_http_json_rpc_client(LOCAL_JSON_RPC_HTTP_URL, bearer_token.as_deref());

    let default_wallet = lotus_client.wallet_default().await.unwrap();
    let constructor_params = ConstructParams {
        parent: Default::default(),
        name: "".to_string(),
        ipc_gateway_addr: 0,
        consensus: ConsensusType::Delegated,
        min_validator_stake: Default::default(),
        min_validators: 0,
        finality_threshold: 0,
        check_period: 0,
        genesis: vec![]
    };

    let subnet_manager = LotusSubnetManager::new(lotus_client);
    let address = subnet_manager.create_subnet(default_wallet, constructor_params).await.unwrap();

    println!("address: {address:}");

    assert!(!address.to_bytes().is_empty())
}