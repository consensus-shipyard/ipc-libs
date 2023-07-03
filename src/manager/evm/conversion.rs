// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Type conversion between evm and fvm

use crate::manager::evm::manager::{
    agent_subnet_to_evm_addresses, ethers_address_to_fil_address, payload_to_evm_address,
};
use anyhow::anyhow;
use ethers::types::U256;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use ipc_gateway::checkpoint::{CheckData, ChildCheck};
use ipc_gateway::{BottomUpCheckpoint, CrossMsg, StorableMsg};
use ipc_sdk::address::IPCAddress;
use ipc_sdk::subnet_id::SubnetID;
use std::str::FromStr;

impl TryFrom<BottomUpCheckpoint> for crate::manager::evm::subnet_contract::BottomUpCheckpoint {
    type Error = anyhow::Error;

    fn try_from(checkpoint: BottomUpCheckpoint) -> Result<Self, Self::Error> {
        // sig field of checkpoint not currently used for checkpoint verification
        let check_data = checkpoint.data;
        crate::manager::evm::subnet_contract::BottomUpCheckpoint::try_from(check_data)
    }
}

impl TryFrom<CheckData> for crate::manager::evm::subnet_contract::BottomUpCheckpoint {
    type Error = anyhow::Error;

    fn try_from(check_data: CheckData) -> Result<Self, Self::Error> {
        let cross_msgs = check_data
            .cross_msgs
            .cross_msgs
            .unwrap_or_default()
            .into_iter()
            .map(|i| {
                crate::manager::evm::subnet_contract::CrossMsg::try_from(i)
                    .map_err(|e| anyhow!("cannot convert cross msg due to: {e:}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let children = check_data
            .children
            .into_iter()
            .map(|i| {
                crate::manager::evm::subnet_contract::ChildCheck::try_from(i)
                    .map_err(|e| anyhow!("cannot convert child check due to: {e:}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let b = crate::manager::evm::subnet_contract::BottomUpCheckpoint {
            source: crate::manager::evm::subnet_contract::SubnetID::try_from(&check_data.source)?,
            epoch: check_data.epoch as u64,
            fee: U256::from_str(&check_data.cross_msgs.fee.atto().to_string())?,
            cross_msgs,
            children,

            // update these two parameters from caller
            prev_hash: [0; 32],
            proof: ethers::core::types::Bytes::default(),
        };
        Ok(b)
    }
}

impl TryFrom<CrossMsg> for crate::manager::evm::subnet_contract::CrossMsg {
    type Error = anyhow::Error;

    fn try_from(value: CrossMsg) -> Result<Self, Self::Error> {
        let c = crate::manager::evm::subnet_contract::CrossMsg {
            wrapped: value.wrapped,
            message: crate::manager::evm::subnet_contract::StorableMsg::try_from(value.msg)
                .map_err(|e| anyhow!("cannot convert storable msg due to: {e:}"))?,
        };
        Ok(c)
    }
}

impl TryFrom<IPCAddress> for crate::manager::evm::subnet_contract::Ipcaddress {
    type Error = anyhow::Error;

    fn try_from(value: IPCAddress) -> Result<Self, Self::Error> {
        Ok(crate::manager::evm::subnet_contract::Ipcaddress {
            subnet_id: crate::manager::evm::subnet_contract::SubnetID::try_from(&value.subnet()?)?,
            raw_address: payload_to_evm_address(value.raw_addr()?.payload())?,
        })
    }
}

impl TryFrom<StorableMsg> for crate::manager::evm::subnet_contract::StorableMsg {
    type Error = anyhow::Error;

    fn try_from(value: StorableMsg) -> Result<Self, Self::Error> {
        log::info!(
            "storable message token amount: {:}, converted: {:?}",
            value.value.atto().to_string(),
            U256::from_str(&value.value.atto().to_string())?
        );

        let c = crate::manager::evm::subnet_contract::StorableMsg {
            // from: crate::manager::evm::subnet_contract::Ipcaddress::try_from(value.from)
            //     .map_err(|e| anyhow!("cannot convert `from` ipc address msg due to: {e:}"))?,
            // to: crate::manager::evm::subnet_contract::Ipcaddress::try_from(value.to)
            //     .map_err(|e| anyhow!("cannot convert `to`` ipc address due to: {e:}"))?,
            from: crate::manager::evm::subnet_contract::Ipcaddress {
                subnet_id: crate::manager::evm::subnet_contract::SubnetID::try_from(
                    &value.from.subnet()?,
                )?,
                raw_address: ethers::types::Address::from_str(
                    "0x1A79385eAd0e873FE0C441C034636D3Edf7014cC",
                )?,
            },
            to: crate::manager::evm::subnet_contract::Ipcaddress {
                subnet_id: crate::manager::evm::subnet_contract::SubnetID::try_from(
                    &value.to.subnet()?,
                )?,
                raw_address: ethers::types::Address::from_str(
                    "0xeC2804Dd9B992C10396b5Af176f06923d984D90e",
                )?,
            },
            value: U256::from_str(&value.value.atto().to_string())
                .map_err(|e| anyhow!("cannot convert value due to: {e:}"))?,
            nonce: value.nonce,
            // FIXME: we might a better way to handle the encoding of methods and params according to the type of message the cross-net message is targetting.
            method: (value.method as u32).to_be_bytes(),
            params: ethers::core::types::Bytes::from(value.params.to_vec()),
        };
        Ok(c)
    }
}

impl TryFrom<ChildCheck> for crate::manager::evm::subnet_contract::ChildCheck {
    type Error = anyhow::Error;

    fn try_from(value: ChildCheck) -> Result<Self, Self::Error> {
        let c = crate::manager::evm::subnet_contract::ChildCheck {
            source: crate::manager::evm::subnet_contract::SubnetID::try_from(&value.source)
                .map_err(|e| anyhow!("cannot convert subnet id due to: {e:}"))?,
            checks: value
                .checks
                .iter()
                .map(|c| {
                    let mut v = [0; 32];
                    // TODO: we should update the solidity contract to use bytes
                    v.copy_from_slice(&c.cid().to_bytes()[0..32]);
                    v
                })
                .collect(),
        };
        Ok(c)
    }
}

impl TryFrom<&SubnetID> for crate::manager::evm::subnet_contract::SubnetID {
    type Error = anyhow::Error;

    fn try_from(subnet: &SubnetID) -> Result<Self, Self::Error> {
        Ok(crate::manager::evm::subnet_contract::SubnetID {
            root: subnet.root_id(),
            route: agent_subnet_to_evm_addresses(subnet)?,
        })
    }
}

impl TryFrom<crate::manager::evm::subnet_contract::FvmAddress> for Address {
    type Error = anyhow::Error;

    fn try_from(
        value: crate::manager::evm::subnet_contract::FvmAddress,
    ) -> Result<Self, Self::Error> {
        let protocol = value.addr_type;
        let addr = match protocol {
            1 => Address::from_bytes(
                &[[1u8].as_slice(), value.payload.to_vec().as_slice()].concat(),
            )?,
            _ => return Err(anyhow!("address not support now")),
        };
        Ok(addr)
    }
}
impl From<Address> for crate::manager::evm::subnet_contract::FvmAddress {
    fn from(value: Address) -> Self {
        crate::manager::evm::subnet_contract::FvmAddress {
            addr_type: value.protocol() as u8,
            payload: ethers::core::types::Bytes::from(value.payload_bytes()),
        }
    }
}

impl TryFrom<crate::manager::evm::gateway::SubnetID> for SubnetID {
    type Error = anyhow::Error;

    fn try_from(value: crate::manager::evm::gateway::SubnetID) -> Result<Self, Self::Error> {
        let children = value
            .route
            .iter()
            .map(ethers_address_to_fil_address)
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(SubnetID::new(value.root, children))
    }
}

impl TryFrom<&SubnetID> for crate::manager::evm::gateway::SubnetID {
    type Error = anyhow::Error;

    fn try_from(subnet: &SubnetID) -> Result<Self, Self::Error> {
        Ok(crate::manager::evm::gateway::SubnetID {
            root: subnet.root_id(),
            route: agent_subnet_to_evm_addresses(subnet)?,
        })
    }
}

impl TryFrom<crate::manager::evm::gateway::Ipcaddress> for IPCAddress {
    type Error = anyhow::Error;

    fn try_from(value: crate::manager::evm::gateway::Ipcaddress) -> Result<Self, Self::Error> {
        let i = IPCAddress::new(
            &SubnetID::try_from(value.subnet_id)?,
            &ethers_address_to_fil_address(&value.raw_address)?,
        )?;
        Ok(i)
    }
}

impl TryFrom<crate::manager::evm::gateway::StorableMsg> for StorableMsg {
    type Error = anyhow::Error;

    fn try_from(value: crate::manager::evm::gateway::StorableMsg) -> Result<Self, Self::Error> {
        let s = StorableMsg {
            from: IPCAddress::try_from(value.from)?,
            to: IPCAddress::try_from(value.to)?,
            method: u32::from_be_bytes(value.method) as MethodNum,
            params: RawBytes::from(value.params.to_vec()),
            value: TokenAmount::from_atto(value.value.as_u128()),
            nonce: value.nonce,
        };
        Ok(s)
    }
}

impl TryFrom<crate::manager::evm::gateway::CrossMsg> for CrossMsg {
    type Error = anyhow::Error;

    fn try_from(value: crate::manager::evm::gateway::CrossMsg) -> Result<Self, Self::Error> {
        let c = CrossMsg {
            wrapped: value.wrapped,
            msg: StorableMsg::try_from(value.message)?,
        };
        Ok(c)
    }
}
