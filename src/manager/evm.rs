// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::{abigen, Signer, SignerMiddleware};
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

use super::subnet::SubnetManager;
use crate::config::subnet::SubnetConfig;
use crate::config::Subnet;
use crate::lotus::message::ipc::SubnetInfo;
use crate::lotus::message::wallet::WalletKeyType;
pub type MiddlewareImpl = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

/// The majority vote percentage for checkpoint submission when creating a subnet.
const SUBNET_MAJORITY_PERCENTAGE: u8 = 60;
const TRANSACTION_RECEIPT_RETRIES: usize = 10;

// Create type bindings for the IPC Solidity contracts
abigen!(Gateway, "contracts/Gateway.json");
abigen!(SubnetContract, "contracts/SubnetActor.json");
abigen!(SubnetRegistry, "contracts/SubnetRegistry.json");

pub struct EthSubnetManager<M: Middleware> {
    eth_client: Arc<M>,
    gateway_contract: Gateway<Arc<M>>,
    registry_contract: SubnetRegistry<Arc<M>>,
}

#[async_trait]
impl<M: Middleware + Send + Sync + 'static> SubnetManager for EthSubnetManager<M> {
    async fn create_subnet(&self, _from: Address, params: ConstructParams) -> Result<Address> {
        self.ensure_same_gateway(&params.ipc_gateway_addr)?;

        let min_validator_stake = params
            .min_validator_stake
            .atto()
            .to_u128()
            .ok_or_else(|| anyhow!("invalid min validator stake"))?;

        log::debug!("calling create subnet for EVM manager");

        let mut route = agent_subnet_to_evm_addresses(&params.parent)?;
        let mut root = vec![ethers::types::Address::from_str(
            "0x0000000000000000000000000000000000000000",
        )?];
        root.append(&mut route);
        log::debug!("root SubnetID as Ethereum type: {root:?}");

        let params = ConstructorParams {
            parent_id: subnet_registry::SubnetID { route: root },
            name: params.name,
            ipc_gateway_addr: (*self.gateway_contract).address(),
            consensus: params.consensus as u64 as u8,
            min_activation_collateral: ethers::types::U256::from(min_validator_stake),
            min_validators: params.min_validators,
            bottom_up_check_period: params.bottomup_check_period as u64,
            top_down_check_period: params.topdown_check_period as u64,
            majority_percentage: SUBNET_MAJORITY_PERCENTAGE,
            genesis: ethers::types::Bytes::default(),
        };

        log::info!("creating subnet on evm with params: {params:?}");

        let call = self.registry_contract.new_subnet_actor(params);
        let pending_tx = call.send().await?;
        // We need the retry to parse the deployment event. At the time of this writing, it's a bug
        // in current FEVM that without the retries, events are not picked up.
        // See https://github.com/filecoin-project/community/discussions/638 for more info and updates.
        let receipt = pending_tx.retries(TRANSACTION_RECEIPT_RETRIES).await?;
        return match receipt {
            Some(r) => {
                for log in r.logs {
                    log::debug!("log: {log:?}");

                    match ethers_contract::parse_log::<subnet_registry::SubnetDeployedFilter>(log) {
                        Ok(subnet_deploy) => {
                            let subnet_registry::SubnetDeployedFilter {
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
        params: JoinParams,
    ) -> Result<()> {
        let collateral = collateral
            .atto()
            .to_u128()
            .ok_or_else(|| anyhow!("invalid min validator stake"))?;

        let address = agent_subnet_to_evm_address(&subnet)?;
        log::info!(
            "interacting with evm subnet contract: {address:} with collateral: {collateral:}"
        );

        let contract = SubnetContract::new(address, self.eth_client.clone());

        let mut txn = contract.join(params.validator_net_addr);
        txn.tx.set_value(collateral);

        txn.send().await?.await?;

        Ok(())
    }

    async fn leave_subnet(&self, subnet: SubnetID, _from: Address) -> Result<()> {
        let address = agent_subnet_to_evm_address(&subnet)?;
        log::info!("leaving evm subnet: {subnet:} at contract: {address:}");

        let contract = SubnetContract::new(address, self.eth_client.clone());
        contract.leave().send().await?.await?;

        Ok(())
    }

    async fn kill_subnet(&self, subnet: SubnetID, _from: Address) -> Result<()> {
        let address = agent_subnet_to_evm_address(&subnet)?;
        log::info!("kill evm subnet: {subnet:} at contract: {address:}");

        let contract = SubnetContract::new(address, self.eth_client.clone());
        contract.kill().send().await?.await?;

        Ok(())
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
        registry_contract: SubnetRegistry<Arc<M>>,
    ) -> Self {
        Self {
            eth_client,
            gateway_contract,
            registry_contract,
        }
    }

    pub fn ensure_same_gateway(&self, gateway: &Address) -> anyhow::Result<()> {
        let evm_gateway_addr = payload_to_evm_address(gateway.payload())?;
        if evm_gateway_addr != (*self.gateway_contract).address() {
            Err(anyhow!("Gateway address not matching with config"))
        } else {
            Ok(())
        }
    }
}

impl EthSubnetManager<MiddlewareImpl> {
    pub fn from_subnet(subnet: &Subnet) -> Result<Self> {
        let url = subnet.rpc_http().clone();
        let auth_token = subnet.auth_token();

        let config = if let SubnetConfig::Evm(config) = &subnet.config {
            config
        } else {
            return Err(anyhow!("not evm config"));
        };

        let provider = if auth_token.is_some() {
            Http::new_with_auth(url, Authorization::Bearer(auth_token.unwrap()))?
        } else {
            Http::new(url)
        };

        let provider = Provider::new(provider);
        let wallet = config.private_key.parse::<LocalWallet>()?;
        let wallet = wallet.with_chain_id(subnet.id.chain_id());

        let gateway_address = payload_to_evm_address(config.gateway_addr.payload())?;
        let registry_address = payload_to_evm_address(config.registry_addr.payload())?;

        let signer = Arc::new(SignerMiddleware::new(provider, wallet));
        let gateway_contract = Gateway::new(gateway_address, Arc::new(signer.clone()));
        let evm_registry_contract = SubnetRegistry::new(registry_address, Arc::new(signer.clone()));

        Ok(Self::new(signer, gateway_contract, evm_registry_contract))
    }
}

/// Convert the ipc SubnetID type to an evm address. It extracts the last address from the Subnet id
/// children and turns it into evm address.
fn agent_subnet_to_evm_address(subnet: &SubnetID) -> Result<ethers::types::Address> {
    let children = subnet.children();
    let ipc_addr = children
        .last()
        .ok_or_else(|| anyhow!("{subnet:} has no child"))?;

    payload_to_evm_address(ipc_addr.payload())
}

/// Convert the ipc SubnetID type to a vec of evm addresses. It extracts all the children addresses
/// in the subnet id and turns them as a vec of evm addresses.
fn agent_subnet_to_evm_addresses(subnet: &SubnetID) -> Result<Vec<ethers::types::Address>> {
    let children = subnet.children();
    children
        .iter()
        .map(|addr| payload_to_evm_address(addr.payload()))
        .collect::<Result<_>>()
}

/// Util function to convert Fil address payload to evm address. Only delegated address is supported.
fn payload_to_evm_address(payload: &Payload) -> Result<ethers::types::Address> {
    match payload {
        Payload::Delegated(delegated) => {
            let slice = delegated.subaddress();
            Ok(ethers::types::Address::from_slice(&slice[0..20]))
        }
        _ => Err(anyhow!("invalid is invalid")),
    }
}

#[cfg(test)]
mod tests {
    use crate::manager::evm::{agent_subnet_to_evm_address, agent_subnet_to_evm_addresses};
    use fvm_shared::address::Address;
    use ipc_sdk::subnet_id::SubnetID;
    use primitives::EthAddress;
    use std::str::FromStr;

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

        let a =
            ethers::types::Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let b =
            ethers::types::Address::from_str("0x2e714a3c385ea88a09998ed74db265dae9853667").unwrap();

        assert_eq!(addrs, vec![a, b]);
    }
}
