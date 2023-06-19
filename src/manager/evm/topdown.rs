//! Top down type conversion

use crate::manager::evm::manager::ethers_address_to_fil_address;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use ipc_gateway::{CrossMsg, StorableMsg};
use ipc_sdk::address::IPCAddress;
use ipc_sdk::subnet_id::SubnetID;

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
