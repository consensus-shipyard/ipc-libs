//! Type conversion between evm and fvm

use crate::manager::evm::manager::{agent_subnet_to_evm_addresses, payload_to_evm_address};
use ethers::types::U256;
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
        let b = crate::manager::evm::subnet_contract::BottomUpCheckpoint {
            source: crate::manager::evm::subnet_contract::SubnetID::try_from(&check_data.source)?,
            epoch: check_data.epoch as u64,
            fee: U256::from_str(&check_data.cross_msgs.fee.atto().to_string())?,
            cross_msgs: vec![],
            children: vec![],

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
            message: crate::manager::evm::subnet_contract::StorableMsg::try_from(value.msg)?,
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
        let c = crate::manager::evm::subnet_contract::StorableMsg {
            from: crate::manager::evm::subnet_contract::Ipcaddress::try_from(value.from)?,
            to: crate::manager::evm::subnet_contract::Ipcaddress::try_from(value.to)?,
            value: ethers::core::types::U256::from_str(&value.value.atto().to_string())?,
            nonce: value.nonce,
            // TODO: we might need a more gerneral method to handle the method
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
            source: crate::manager::evm::subnet_contract::SubnetID::try_from(&value.source)?,
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
