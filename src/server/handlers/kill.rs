// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet handler and parameters

use crate::manager::SubnetManager;
use crate::server::handlers::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct KillSubnetParams {
    pub subnet: String,
    pub from: Option<String>,
}

/// The create subnet json rpc method handler.
pub(crate) struct KillSubnetHandler {
    pool: Arc<SubnetManagerPool>,
}

impl KillSubnetHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for KillSubnetHandler {
    type Request = KillSubnetParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let conn = match self.pool.get(&request.subnet) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet = SubnetID::from_str(&request.subnet)?;
        let from = match request.from {
            Some(addr) => Address::from_str(&addr)?,
            None => conn.subnet().accounts[0],
        };

        conn.manager().kill_subnet(subnet, from).await
    }
}
