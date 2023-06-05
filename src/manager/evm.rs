// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::{abigen, Signer, SignerMiddleware, H160};
use ethers::providers::{Authorization, Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Wallet};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConstructParams, JoinParams};
use num_traits::ToPrimitive;
use primitives::EthAddress;

use crate::config::Subnet;
use crate::lotus::message::ipc::SubnetInfo;
use crate::lotus::message::wallet::WalletKeyType;

use super::subnet::SubnetManager;

pub type MiddlewareImpl = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

// Create type bindings for the IPC Solidity contracts
abigen!(Gateway, "contracts/Gateway.json");
abigen!(SubnetContract, "contracts/SubnetActor.json");
abigen!(IPCRegistry, "contracts/IPCRegistry.json");

pub struct EthSubnetManager<M: Middleware> {
    eth_client: Arc<M>,
    gateway_contract: Gateway<Arc<M>>,
    registry_contract: IPCRegistry<Arc<M>>,
}

#[async_trait]
impl<M: Middleware + Send + Sync + 'static> SubnetManager for EthSubnetManager<M> {
    async fn create_subnet(&self, _from: Address, params: ConstructParams) -> Result<Address> {
        let parent = params.parent.clone();
        let min_validator_stake = params
            .min_validator_stake
            .atto()
            .to_u64()
            .ok_or_else(|| anyhow!("invalid min validator stake"))?;

        let params = IsubnetActorConstructorParams {
            // TODO: replace this with parent
            parent_id: ipc_registry::SubnetID {
                route: vec![ethers::types::H160::from_str(
                    "0x0000000000000000000000000000000000000000",
                )?],
            },
            name: params.name,
            // TODO: use ipc sdk address
            ipc_gateway_addr: ethers::types::H160::from_str(
                "0x008Ee541Cc66D2A91c3624Da943406D719CF42EF",
            )?,
            consensus: params.consensus as u64 as u8,
            min_activation_collateral: ethers::types::U256::from(min_validator_stake as u128),
            min_validators: params.min_validators,
            bottom_up_check_period: params.bottomup_check_period as u64,
            top_down_check_period: params.topdown_check_period as u64,
            // TODO: update this variable properly
            majority_percentage: 50,
            genesis: ethers::types::Bytes::default(),
        };
        log::info!("creating subnet: {params:?}");

        let mut call = self.registry_contract.new_subnet_actor(params);

        log::debug!("sending create transaction");
        let r = call.send().await?.await?;
        return match r {
            Some(r) => {
                for log in r.logs {
                    log::debug!("log: {log:?}");

                    match ethers_contract::parse_log::<ipc_registry::SubnetDeployedFilter>(log) {
                        Ok(subnet_deploy) => {
                            let ipc_registry::SubnetDeployedFilter {
                                subnet_addr,
                                subnet_id,
                            } = subnet_deploy;

                            log::debug!("subnet with id {subnet_id:?} deployed at {subnet_addr:?}");

                            let eth_addr = EthAddress::from_str(&subnet_addr.to_string())?;
                            return Ok(Address::from(eth_addr));
                        }
                        Err(_) => {
                            log::debug!("not of event subnet actor deployed, continue");
                            continue;
                        }
                    }
                }
                Err(anyhow!("no logs receipt"))
            }
            None => {
                Err(anyhow!("no receipt to event, txn not successful"))
            }
        }
    }

    async fn join_subnet(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        params: JoinParams,
    ) -> Result<()> {
        // TODO: Convert IPC SubnetID to evm SubnetID
        let evm_subnet_id = ipc_registry::SubnetID::default();
        let address = self
            .registry_contract
            .subnet_address(evm_subnet_id)
            .call()
            .await?;

        let contract = SubnetContract::new(address, self.eth_client.clone());
        // TODO: check how to send `collateral` as value
        contract.join().send().await?.await?;
        Ok(())
    }

    async fn leave_subnet(&self, _subnet: SubnetID, _from: Address) -> Result<()> {
        todo!()
    }

    async fn kill_subnet(&self, _subnet: SubnetID, _from: Address) -> Result<()> {
        todo!()
    }

    async fn list_child_subnets(
        &self,
        _gateway_addr: Address,
    ) -> Result<HashMap<SubnetID, SubnetInfo>> {
        todo!()
    }

    async fn fund(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _from: Address,
        _amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        todo!()
    }

    async fn release(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _from: Address,
        _amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        todo!()
    }

    async fn propagate(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _from: Address,
        _postbox_msg_cid: Cid,
    ) -> Result<()> {
        todo!()
    }

    async fn set_validator_net_addr(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _validator_net_addr: String,
    ) -> Result<()> {
        todo!()
    }

    async fn whitelist_propagator(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _postbox_msg_cid: Cid,
        _from: Address,
        _to_add: Vec<Address>,
    ) -> Result<()> {
        todo!()
    }

    /// Send value between two addresses in a subnet
    async fn send_value(&self, _from: Address, _to: Address, _amount: TokenAmount) -> Result<()> {
        todo!()
    }

    async fn wallet_new(&self, _key_type: WalletKeyType) -> Result<Address> {
        todo!()
    }

    async fn wallet_list(&self) -> Result<Vec<Address>> {
        todo!()
    }

    async fn wallet_balance(&self, _address: &Address) -> Result<TokenAmount> {
        todo!()
    }

    async fn last_topdown_executed(&self, _gateway_addr: &Address) -> Result<ChainEpoch> {
        todo!()
    }

    async fn list_checkpoints(
        &self,
        _subnet_id: SubnetID,
        _from_epoch: ChainEpoch,
        _to_epoch: ChainEpoch,
    ) -> Result<Vec<BottomUpCheckpoint>> {
        todo!()
    }
}

impl<M: Middleware + Send + Sync> EthSubnetManager<M> {
    pub fn new(
        eth_client: Arc<M>,
        gateway_contract: Gateway<Arc<M>>,
        registry_contract: IPCRegistry<Arc<M>>,
    ) -> Self {
        Self {
            eth_client,
            gateway_contract,
            registry_contract,
        }
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
            let provider = if auth_token.is_some() {
                Http::new_with_auth(
                    url,
                    Authorization::Bearer(auth_token.unwrap_or_default().to_string()),
                )?
            } else {
                Http::new(url)
            };

            let provider = Provider::new(provider);
            let wallet = priv_key.parse::<LocalWallet>()?;
            let wallet = wallet.with_chain_id(subnet.chain_id.unwrap_or_default());

            let evm_gateway_address = subnet
                .evm_gateway_address
                .as_ref()
                .ok_or_else(|| anyhow!("evm gateway address not defined"))?;
            let gateway_address = H160::from_str(evm_gateway_address)?;

            let evm_registry_address = subnet
                .evm_registry_address
                .as_ref()
                .ok_or_else(|| anyhow!("evm registry address not defined"))?;
            let evm_registry_address = H160::from_str(evm_registry_address)?;

            let signer = Arc::new(SignerMiddleware::new(provider, wallet));
            let gateway_contract = Gateway::new(gateway_address, Arc::new(signer.clone()));
            let evm_registry_contract =
                IPCRegistry::new(evm_registry_address, Arc::new(signer.clone()));
            return Ok(Self::new(signer, gateway_contract, evm_registry_contract));
        }

        Err(anyhow!(
            "no ethereum-compatible private key provided in config"
        ))
    }
}

// fn evm_id_to_address(evm_subnet: ipc_registry::SubnetID) -> Result<Address> {
//     // TODO: maybe do a check to ensure the parent subnet id have common parents with new child id
//     let mut children = vec![];
//     let addr = evm_subnet
//         .route
//         .last()
//         .ok_or_else(anyhow!("invdalid evm address passed"))?;
//     let eth_addr = EthAddress::from_str(addr.to_string())?;
//     Ok(Address::from(eth_addr))
// }
//
// fn evm_id_to_agent_id(parent: &SubnetID, evm_subnet: ipc_registry::SubnetID) -> Result<SubnetID> {
//     // TODO: maybe do a check to ensure the parent subnet id have common parents with new child id
//     let mut children = vec![];
//     for addr in evm_subnet.route.iter() {
//         let eth_addr = EthAddress::from_str(addr.to_string())?;
//         children.push(Address::from(eth_addr));
//     }
//     Ok(SubnetID::new(parent.root_id(), children))
// }
