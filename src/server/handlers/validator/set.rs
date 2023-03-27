// Copyright 2022-2023 Protocol Labs
//! Set the memberships of the validators.

use crate::config::DEFAULT_IPC_GATEWAY_ADDR;
use crate::lotus::message::ipc::ValidatorSet;
use crate::manager::SubnetManager;
use crate::server::subnet::SubnetManagerPool;
use crate::server::{check_subnet, parse_from, JsonRPCRequestHandler};
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetMembershipParams {
    pub subnet: String,
    // TODO: we should take the gateway address from config
    pub gateway_addr: Option<String>,
    pub validator_set: ValidatorSet,
    pub from: Option<String>,
}

/// Set the list of validators in the gateway of a subnet.
pub(crate) struct SetMembershipHandler {
    pool: Arc<SubnetManagerPool>,
}

impl SetMembershipHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for SetMembershipHandler {
    type Request = SetMembershipParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let conn = match self.pool.get(&request.subnet) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let from = parse_from(subnet_config, request.from)?;
        let gateway_addr = if let Some(addr) = request.gateway_addr.as_ref() {
            Address::from_str(addr)?
        } else {
            Address::new_id(DEFAULT_IPC_GATEWAY_ADDR)
        };
        let validator_set = ipc_sdk::ValidatorSet::try_from(request.validator_set)?;

        conn.manager()
            .set_memberships(from, gateway_addr, validator_set)
            .await
    }
}
