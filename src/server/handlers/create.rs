//! Create subnet handler and parameters

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::ConsensusType;
use serde::Deserialize;
use crate::config::{Config, Subnet};
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

#[derive(Debug, Deserialize)]
pub struct CreateSubnetResponse {
    /// The address of the created subnet
    pub address: String
}

/// The create subnet json rpc method handler.
pub struct CreateSubnetHandler {
    manager: Arc<SubnetManagerShared>,
}

#[async_trait]
impl JsonRPCRequestHandler for CreateSubnetHandler {
    type Request = CreateSubnetParams;
    type Response = CreateSubnetResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        self.manager.create_subnet()
    }
}
