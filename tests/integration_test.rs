use fvm_shared::address::{Network, set_current_network};
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::{ROOTNET_ID};
use ipc_subnet_actor::{ConsensusType, ConstructParams};
use ipc_agent::lotus::LotusClient;
use ipc_agent::manager::{LotusSubnetManager, SubnetManager};

mod setup;

/// To run this test:
/// ```shell
/// export IPC_JSON_RPC_TEST_HTTP_URL="http://127.0.0.1:<Your Node Port>/rpc/v0"
/// export IPC_JSON_RPC_TEST_BEARER_TOKEN=<Your admin token>
/// RUST_LOG=debug cargo test --test '*'
/// ```
#[tokio::test]
#[ignore]
async fn test_create_subnet_actor() {
    let bearer_token = std::env::var("IPC_JSON_RPC_TEST_BEARER_TOKEN").ok();
    let http_url = std::env::var("IPC_JSON_RPC_TEST_HTTP_URL").unwrap();
    let lotus_client = setup::lotus_http_json_rpc_client(&http_url, bearer_token.as_deref());

    set_current_network(Network::Testnet);

    let default_wallet = lotus_client.wallet_default().await.unwrap();
    let constructor_params = ConstructParams {
        parent: ROOTNET_ID.clone(),
        name: "test".to_string(),
        ipc_gateway_addr: 64,
        consensus: ConsensusType::Mir,
        min_validator_stake: TokenAmount::from_atto(1),
        min_validators: 0,
        finality_threshold: 2,
        check_period: 10,
        genesis: vec![]
    };

    let subnet_manager = LotusSubnetManager::new(lotus_client);
    let address = subnet_manager.create_subnet(default_wallet, constructor_params).await.unwrap();

    println!("address: {address:}");

    assert!(!address.to_bytes().is_empty())
}