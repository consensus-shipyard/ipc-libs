use std::str::FromStr;
use fvm_shared::address::{Network, set_current_network};
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::{ROOTNET_ID, SubnetID};
use ipc_subnet_actor::{ConsensusType, ConstructParams, JoinParams};
use ipc_agent::jsonrpc::JsonRpcClientImpl;
use ipc_agent::lotus::{LotusClient, LotusJsonRPCClient};
use ipc_agent::manager::{LotusSubnetManager, SubnetManager};

/// To run this test:
/// ```shell
/// export IPC_JSON_RPC_TEST_HTTP_URL="http://127.0.0.1:<Your Node Port>/rpc/v0"
/// export IPC_JSON_RPC_TEST_BEARER_TOKEN=<Your admin token>
/// RUST_LOG=debug cargo test --test '*'
/// ```
#[tokio::test]
#[ignore]
async fn test_create_subnet_actor() {
    env_logger::init();

    let bearer_token = std::env::var("IPC_JSON_RPC_TEST_BEARER_TOKEN").ok();
    let http_url = std::env::var("IPC_JSON_RPC_TEST_HTTP_URL").unwrap();
    let lotus_client = lotus_http_json_rpc_client(&http_url, bearer_token.as_deref());

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

/// To run this test:
/// ```shell
/// export IPC_JSON_RPC_TEST_HTTP_URL="http://127.0.0.1:<Your Node Port>/rpc/v0"
/// export IPC_JSON_RPC_TEST_BEARER_TOKEN=<Your admin token>
/// RUST_LOG=debug cargo test --test '*'
/// ```
#[tokio::test]
#[ignore]
async fn test_join_subnet_actor() {
    env_logger::init();

    let bearer_token = std::env::var("IPC_JSON_RPC_TEST_BEARER_TOKEN").ok();
    let http_url = std::env::var("IPC_JSON_RPC_TEST_HTTP_URL").unwrap();
    let subnet_id_string = std::env::var("IPC_JSON_RPC_TEST_SUBNET_ID").unwrap();
    let lotus_client = lotus_http_json_rpc_client(&http_url, bearer_token.as_deref());

    set_current_network(Network::Testnet);

    let default_wallet = lotus_client.wallet_default().await.unwrap();

    let subnet = SubnetID::from_str(&subnet_id_string).unwrap();
    let collateral = TokenAmount::from_atto(10);
    let params = JoinParams { validator_net_addr: "test".to_string() };

    let subnet_manager = LotusSubnetManager::new(lotus_client);
    subnet_manager.join_subnet(subnet, default_wallet, collateral, params).await.unwrap();
}

/// To run this test:
/// ```shell
/// export IPC_JSON_RPC_TEST_HTTP_URL="http://127.0.0.1:<Your Node Port>/rpc/v0"
/// export IPC_JSON_RPC_TEST_BEARER_TOKEN=<Your admin token>
/// RUST_LOG=debug cargo test --test '*'
/// ```
#[tokio::test]
#[ignore]
async fn test_leave_subnet_actor() {
    env_logger::init();

    let bearer_token = std::env::var("IPC_JSON_RPC_TEST_BEARER_TOKEN").ok();
    let http_url = std::env::var("IPC_JSON_RPC_TEST_HTTP_URL").unwrap();
    let subnet_id_string = std::env::var("IPC_JSON_RPC_TEST_SUBNET_ID").unwrap();
    let lotus_client = lotus_http_json_rpc_client(&http_url, bearer_token.as_deref());

    set_current_network(Network::Testnet);

    let default_wallet = lotus_client.wallet_default().await.unwrap();
    let subnet = SubnetID::from_str(&subnet_id_string).unwrap();

    let subnet_manager = LotusSubnetManager::new(lotus_client);
    subnet_manager.leave_subnet(subnet, default_wallet).await.unwrap();
}

/// To run this test:
/// ```shell
/// export IPC_JSON_RPC_TEST_HTTP_URL="http://127.0.0.1:<Your Node Port>/rpc/v0"
/// export IPC_JSON_RPC_TEST_BEARER_TOKEN=<Your admin token>
/// RUST_LOG=debug cargo test --test '*'
/// ```
#[tokio::test]
#[ignore]
async fn test_kill_subnet_actor() {
    env_logger::init();

    let bearer_token = std::env::var("IPC_JSON_RPC_TEST_BEARER_TOKEN").ok();
    let http_url = std::env::var("IPC_JSON_RPC_TEST_HTTP_URL").unwrap();
    let subnet_id_string = std::env::var("IPC_JSON_RPC_TEST_SUBNET_ID").unwrap();
    let lotus_client = lotus_http_json_rpc_client(&http_url, bearer_token.as_deref());

    set_current_network(Network::Testnet);

    let default_wallet = lotus_client.wallet_default().await.unwrap();
    let subnet = SubnetID::from_str(&subnet_id_string).unwrap();

    let subnet_manager = LotusSubnetManager::new(lotus_client);
    subnet_manager.kill_subnet(subnet, default_wallet).await.unwrap();
}

fn http_json_rpc(url: &str, bearer_token: Option<&str>) -> JsonRpcClientImpl {
    JsonRpcClientImpl::new(url.parse().unwrap(), bearer_token)
}

fn lotus_http_json_rpc_client(url: &str, bearer_token: Option<&str>) -> LotusJsonRPCClient<JsonRpcClientImpl> {
    let json_rpc = http_json_rpc(url, bearer_token);
    LotusJsonRPCClient::new(json_rpc)
}