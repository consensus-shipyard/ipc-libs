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
use fvm_shared::address::Payload;
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
        let min_validator_stake = params
            .min_validator_stake
            .atto()
            .to_u64()
            .ok_or_else(|| anyhow!("invalid min validator stake"))?;

        let params = IsubnetActorConstructorParams {
            // TODO: replace this with parent
            parent_id: ipc_registry::SubnetID {
                route: agent_subnet_to_evm_addresses(&params.parent)?,
            },
            name: params.name,
            ipc_gateway_addr: (*self.gateway_contract).address(),
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

        let call = self.registry_contract.new_subnet_actor(params);
        let pending_tx = call.send().await?;
        let receipt = pending_tx.retries(10).await?;
        return match receipt {
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

                            // subnet_addr.to_string() returns a summary of the actual Ethereum address, not
                            // usable in the actual code.
                            let subnet_addr = format!("{subnet_addr:?}");
                            log::debug!("raw subnet addr: {subnet_addr:}");

                            let eth_addr = EthAddress::from_str(&subnet_addr)?;
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
            None => Err(anyhow!("no receipt to event, txn not successful")),
        };
    }

    async fn join_subnet(
        &self,
        subnet: SubnetID,
        _from: Address,
        collateral: TokenAmount,
        _params: JoinParams,
    ) -> Result<()> {
        let collateral = collateral
            .atto()
            .to_u64()
            .ok_or_else(|| anyhow!("invalid min validator stake"))?;

        let address = agent_subnet_to_evm_address(&subnet)?;
        log::info!(
            "interacting with evm subnet contract: {address:} with collateral: {collateral:}"
        );

        let contract = SubnetContract::new(address, self.eth_client.clone());

        let mut txn = contract.join();
        txn.tx.set_value(collateral);

        txn.send().await?.await?;

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

fn agent_subnet_to_evm_address(subnet: &SubnetID) -> Result<ethers::types::Address> {
    let children = subnet.children();
    let ipc_addr = children
        .last()
        .ok_or_else(|| anyhow!("{subnet:} has no child"))?;

    payload_to_evm_address(ipc_addr.payload())
}

fn payload_to_evm_address(payload: &Payload) -> Result<ethers::types::Address> {
    match payload {
        Payload::Delegated(delegated) => {
            let slice = delegated.subaddress();
            Ok(ethers::types::Address::from_slice(&slice[0..20]))
        }
        _ => Err(anyhow!("invalid is invalid")),
    }
}

fn agent_subnet_to_evm_addresses(subnet: &SubnetID) -> Result<Vec<ethers::types::Address>> {
    let children = subnet.children();
    children
        .iter()
        .map(|addr| payload_to_evm_address(addr.payload()))
        .collect::<Result<_>>()
}

#[cfg(test)]
mod tests {
    use crate::manager::evm::{agent_subnet_to_evm_address, agent_subnet_to_evm_addresses};
    use fvm_shared::address::Address;
    use ipc_sdk::subnet_id::SubnetID;
    use primitives::EthAddress;
    use std::str::FromStr;

    #[test]
    fn test_addr_convert() {
        let eth_addr = EthAddress::from_str("0x0b0d23d88d21527049232a3248fe31949d90b03b").unwrap();

        let addr = Address::from(eth_addr);

        println!("{addr:?}");
    }

    #[test]
    fn test_agent_subnet_to_evm_address() {
        let addr = Address::from_str("f410ffzyuupbyl2uiucmzr3lu3mtf3luyknthaz4xsrq").unwrap();
        let id = SubnetID::new(0, vec![addr]);

        let eth = agent_subnet_to_evm_address(&id).unwrap();
        assert_eq!(
            format!("{eth:?}"),
            "0x2e714a3c385ea88a09998ed74db265dae9853667"
        );
    }

    #[test]
    fn test_agent_subnet_to_evm_addresses() {
        let eth_addr = EthAddress::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let addr = Address::from(eth_addr);
        let addr2 = Address::from_str("f410ffzyuupbyl2uiucmzr3lu3mtf3luyknthaz4xsrq").unwrap();

        let id = SubnetID::new(0, vec![addr, addr2]);

        let addrs = agent_subnet_to_evm_addresses(&id).unwrap();

        let a = ethers::types::Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let b = ethers::types::Address::from_str("0x2e714a3c385ea88a09998ed74db265dae9853667").unwrap();

        assert_eq!(addrs, vec![a, b]);
    }
}
