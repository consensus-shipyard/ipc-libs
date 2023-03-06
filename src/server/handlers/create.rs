//! Create subnet handler and parameters

use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConsensusType, ConstructParams};
use serde::{Deserialize, Serialize};
use crate::config::IPC_GATEWAY_ADDR;
use crate::jsonrpc::JsonRpcClient;
use crate::manager::SubnetManager;
use crate::server::handlers::subnet::SubnetManagerPool;
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
pub(crate) struct CreateSubnetHandler<T: JsonRpcClient> {
    shared: Arc<SubnetManagerPool<T>>,
}

impl <T: JsonRpcClient> CreateSubnetHandler<T> {
    pub(crate) fn new(shared: Arc<SubnetManagerPool<T>>) -> Self {
        Self { shared }
    }
}

#[async_trait]
impl <T: JsonRpcClient + Send + Sync> JsonRPCRequestHandler for CreateSubnetHandler<T> {
    type Request = CreateSubnetParams;
    type Response = CreateSubnetResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let parent = &request.parent;

        if !self.shared.contains_subnet(parent) {
            return Err(anyhow!("target parent subnet not found"));
        }

        // this is safe to unwrap as we ensure this key exists.
        let bundle = self.shared.get(parent).unwrap();

        let constructor_params = ConstructParams {
            parent: SubnetID::from_str(parent)?,
            name: request.name,
            ipc_gateway_addr: IPC_GATEWAY_ADDR,
            consensus: ConsensusType::Mir,
            min_validator_stake: TokenAmount::from_atto(request.min_validator_stake),
            min_validators: request.min_validators,
            finality_threshold: request.finality_threshold,
            check_period: request.check_period,
            // TODO: we load from file?
            genesis: vec![]
        };

        let created_subnet_addr = bundle.manager().create_subnet(
            bundle.subnet().accounts[0],
            constructor_params
        ).await?;

        Ok(CreateSubnetResponse{ address: created_subnet_addr.to_string() })
    }
}
