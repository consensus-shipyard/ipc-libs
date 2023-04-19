use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletListParams {
    pub subnet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletListResponse {
    pub addresses: Vec<String>,
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletListHandler {
    pool: Arc<SubnetManagerPool>,
}

impl WalletListHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletListHandler {
    type Request = WalletListParams;
    type Response = WalletListResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let address = conn.manager().wallet_list().await?;
        Ok(WalletListResponse {
            addresses: address.iter().map(|e| e.to_string()).collect(),
        })
    }
}
