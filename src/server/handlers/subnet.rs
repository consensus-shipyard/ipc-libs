use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::str::FromStr;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConsensusType, ConstructParams};
use once_cell::sync::OnceCell;
use crate::config::Subnet;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::lotus::client::LotusJsonRPCClient;
use crate::manager::{LotusSubnetManager, SubnetManager};
use crate::server::handlers::create::{CreateSubnetParams, CreateSubnetResponse};

/// The json rpc subnet manager wrapper struct. This struct can be shared by all the subnet methods.
/// In this case, there is no need to re-init the same SubnetManager for different methods.
pub(crate) struct SubnetManagerShared {
    subnets: HashMap<String, Subnet>,
    manager: HashMap<String, OnceCell<LotusSubnetManager<JsonRpcClientImpl>>>,
}

impl SubnetManagerShared {
    pub fn new(subnets: HashMap<String, Subnet>) -> Self {
        let mut manager = HashMap::new();
        subnets.keys().for_each(|subnet| {
            manager.insert(subnet.clone(), OnceCell::new());
        });
        Self { subnets, manager }
    }

    pub async fn create_subnet(&self, params: CreateSubnetParams) -> Result<CreateSubnetResponse> {
        let parent = &params.parent;

        let pair = self.get_manager_and_gateway(parent);
        if pair.is_none() {
            return Err(anyhow!("target parent subnet not found"));
        }

        let (manager, gateway_addr) = pair.unwrap();

        let constructor_params = ConstructParams {
            parent: SubnetID::from_str(parent)?,
            name: params.name,
            ipc_gateway_addr: gateway_addr,
            consensus: ConsensusType::Mir,
            min_validator_stake: TokenAmount::from_atto(params.min_validator_stake),
            min_validators: params.min_validators,
            finality_threshold: params.finality_threshold,
            check_period: params.check_period,
            // TODO: we load from file?
            genesis: vec![]
        };
        // this is safe to unwrap as we ensure this key exists.
        let subnet = self.subnets.get(parent).unwrap();

        let created_subnet_addr = manager.create_subnet(
            subnet.accounts[0].clone(),
            constructor_params
        ).await?;

        Ok(CreateSubnetResponse{ address: created_subnet_addr.to_string() })
    }

    fn get_manager_and_gateway(&self, subnet_str: &String) -> Option<(&LotusSubnetManager<JsonRpcClientImpl>, u64)> {
        if !self.subnets.contains_key(subnet_str) {
            return None;
        }

        let subnet = self.subnets.get(subnet_str).unwrap();
        let manager_cell = self.manager.get(subnet_str).unwrap();
        let manager = manager_cell.get_or_init(|| {
            let json_rpc_client = JsonRpcClientImpl::new(
                subnet.jsonrpc_api_http.clone(),
                subnet.auth_token.clone().as_deref(),
            );
            let lotus_client = LotusJsonRPCClient::new(json_rpc_client);
            LotusSubnetManager::new(lotus_client)
        });
        Some((manager, subnet.gateway_addr))
    }
}
