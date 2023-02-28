//! Create subnet handler and parameters

use async_trait::async_trait;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConsensusType, ConstructParams};
use serde::{Deserialize, Serialize};
use crate::manager::SubnetManager;
use crate::server::handlers::subnet::SubnetManagerShared;
use crate::server::JsonRPCRequestHandler;

#[derive(Debug, Deserialize)]
pub struct CreateSubnetParams {
    pub parent: String,
    pub name: String,
    pub min_validator_stake: u64,
    pub min_validators: u64,
    pub finality_threshold: ChainEpoch,
    pub check_period: ChainEpoch,
}

#[derive(Debug, Serialize)]
pub struct CreateSubnetResponse {
    /// The address of the created subnet
    pub address: String
}

/// The create subnet json rpc method handler.
pub(crate) struct CreateSubnetHandler {
    shared: Arc<SubnetManagerShared>,
}

impl CreateSubnetHandler {
    pub(crate) fn new(shared: Arc<SubnetManagerShared>) -> Self {
        Self { shared }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for CreateSubnetHandler {
    type Request = CreateSubnetParams;
    type Response = CreateSubnetResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let parent = &request.parent;

        let pair = self.shared.get_manager_and_gateway(parent);
        if pair.is_none() {
            return Err(anyhow!("target parent subnet not found"));
        }

        let (manager, gateway_addr) = pair.unwrap();

        let constructor_params = ConstructParams {
            parent: SubnetID::from_str(parent)?,
            name: request.name,
            ipc_gateway_addr: gateway_addr,
            consensus: ConsensusType::Mir,
            min_validator_stake: TokenAmount::from_atto(request.min_validator_stake),
            min_validators: request.min_validators,
            finality_threshold: request.finality_threshold,
            check_period: request.check_period,
            // TODO: we load from file?
            genesis: vec![]
        };

        // this is safe to unwrap as we ensure this key exists.
        let subnet = self.shared.get_subnet(parent).unwrap();

        let created_subnet_addr = manager.create_subnet(
            subnet.accounts[0],
            constructor_params
        ).await?;

        Ok(CreateSubnetResponse{ address: created_subnet_addr.to_string() })
    }
}
