// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet handler and parameters

use crate::config::DEFAULT_IPC_GATEWAY_ADDR;
use crate::jsonrpc::JsonRpcClient;
use crate::manager::SubnetManager;
use crate::server::handlers::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConsensusType, ConstructParams};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

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
    pub address: String,
}

/// The create subnet json rpc method handler.
pub(crate) struct CreateSubnetHandler<T: JsonRpcClient> {
    pool: Arc<SubnetManagerPool<T>>,
}

impl<T: JsonRpcClient> CreateSubnetHandler<T> {
    pub(crate) fn new(pool: Arc<SubnetManagerPool<T>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> JsonRPCRequestHandler for CreateSubnetHandler<T> {
    type Request = CreateSubnetParams;
    type Response = CreateSubnetResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let parent = &request.parent;

        if !self.pool.contains_subnet(parent) {
            return Err(anyhow!("target parent subnet not found"));
        }

        // this is safe to unwrap as we ensure this key exists.
        let conn = self.pool.get(parent).unwrap();

        let constructor_params = ConstructParams {
            parent: SubnetID::from_str(parent)?,
            name: request.name,
            ipc_gateway_addr: DEFAULT_IPC_GATEWAY_ADDR,
            consensus: ConsensusType::Mir,
            min_validator_stake: TokenAmount::from_atto(request.min_validator_stake),
            min_validators: request.min_validators,
            finality_threshold: request.finality_threshold,
            check_period: request.check_period,
            genesis: vec![],
        };

        let created_subnet_addr = conn
            .manager()
            .create_subnet(conn.subnet().accounts[0], constructor_params)
            .await?;

        Ok(CreateSubnetResponse {
            address: created_subnet_addr.to_string(),
        })
    }
}
