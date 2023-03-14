// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Release operation in the gateway actor

use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseParams {
    pub subnet: String,
    pub from: Option<String>,
    pub amount: u64,
}

/// The Release json rpc method handler.
pub(crate) struct ReleaseHandler {
    pool: Arc<SubnetManagerPool>,
}

impl ReleaseHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for ReleaseHandler {
    type Request = ReleaseParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let conn = match self.pool.get(&request.subnet) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet = SubnetID::from_str(&request.subnet)?;
        let amount = TokenAmount::from_whole(request.amount);
        let from = match request.from {
            Some(addr) => Address::from_str(&addr)?,
            None => conn.subnet().accounts[0],
        };

        conn.manager().release(subnet, from, amount).await
    }
}
