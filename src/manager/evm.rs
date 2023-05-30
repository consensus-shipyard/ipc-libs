// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::{abigen, Signer, SignerMiddleware};
use ethers::providers::{Authorization, Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Wallet};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConstructParams, JoinParams};

use crate::config::Subnet;
use crate::lotus::message::ipc::SubnetInfo;
use crate::lotus::message::wallet::WalletKeyType;

use super::subnet::SubnetManager;

pub type MiddlewareImpl = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

// Create type bindings for the IPC Solidity contracts
abigen!(Gateway, "contracts/Gateway.json");
abigen!(SubnetActor, "contracts/SubnetActor.json");

pub struct EthSubnetManager<M: Middleware> {
    eth_client: Arc<M>,
}

#[async_trait]
impl<M: Middleware + Send + Sync> SubnetManager for EthSubnetManager<M> {
    async fn create_subnet(&self, from: Address, params: ConstructParams) -> Result<Address> {
        todo!()
    }

    async fn join_subnet(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        params: JoinParams,
    ) -> Result<()> {
        todo!()
    }

    async fn leave_subnet(&self, subnet: SubnetID, from: Address) -> Result<()> {
        todo!()
    }

    async fn kill_subnet(&self, subnet: SubnetID, from: Address) -> Result<()> {
        todo!()
    }

    async fn list_child_subnets(
        &self,
        gateway_addr: Address,
    ) -> Result<HashMap<SubnetID, SubnetInfo>> {
        todo!()
    }

    async fn fund(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        todo!()
    }

    async fn release(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        todo!()
    }

    async fn propagate(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        postbox_msg_cid: Cid,
    ) -> Result<()> {
        todo!()
    }

    async fn set_validator_net_addr(
        &self,
        subnet: SubnetID,
        from: Address,
        validator_net_addr: String,
    ) -> Result<()> {
        todo!()
    }

    async fn whitelist_propagator(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        postbox_msg_cid: Cid,
        from: Address,
        to_add: Vec<Address>,
    ) -> Result<()> {
        todo!()
    }

    /// Send value between two addresses in a subnet
    async fn send_value(&self, from: Address, to: Address, amount: TokenAmount) -> Result<()> {
        todo!()
    }

    async fn wallet_new(&self, key_type: WalletKeyType) -> Result<Address> {
        todo!()
    }

    async fn wallet_list(&self) -> Result<Vec<Address>> {
        todo!()
    }

    async fn wallet_balance(&self, address: &Address) -> Result<TokenAmount> {
        todo!()
    }

    async fn last_topdown_executed(&self, gateway_addr: &Address) -> Result<ChainEpoch> {
        todo!()
    }

    async fn list_checkpoints(
        &self,
        subnet_id: SubnetID,
        from_epoch: ChainEpoch,
        to_epoch: ChainEpoch,
    ) -> Result<Vec<BottomUpCheckpoint>> {
        todo!()
    }
}

impl<M: Middleware + Send + Sync> EthSubnetManager<M> {
    pub fn new(eth_client: Arc<M>) -> Self {
        Self { eth_client }
    }
}

impl EthSubnetManager<MiddlewareImpl> {
    pub fn from_subnet(subnet: &Subnet) -> Result<Self> {
        let url = subnet.jsonrpc_api_http.clone();
        let auth_token = subnet.auth_token.as_deref();
        if !subnet.evm {
            return Err(anyhow!("not an evm subnet"));
        }

        if let Some(priv_key) = subnet.eth_private_key.clone() {
            let provider = Http::new_with_auth(
                url,
                Authorization::Bearer(auth_token.unwrap_or_default().to_string()),
            )?;
            let provider = Provider::new(provider);
            let wallet = priv_key.parse::<LocalWallet>()?;
            let wallet = wallet.with_chain_id(subnet.chain_id.unwrap_or_default());

            let signer = Arc::new(SignerMiddleware::new(provider, wallet));
            return Ok(Self::new(signer));
        }

        Err(anyhow!(
            "no ethereum-compatible private key provided in config"
        ))
    }
}
