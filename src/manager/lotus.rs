use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConstructParams, JoinParams};

use super::{SubnetInfo, SubnetManager};

pub struct LotusSubnetManager {}

#[async_trait]
impl SubnetManager for LotusSubnetManager {
    fn create_subnet(
        _parent: SubnetID,
        _from: Address,
        _params: ConstructParams,
    ) -> Result<Address> {
        panic!("not implemented")
    }

    fn join_subnet(
        _subnet: SubnetID,
        _from: Address,
        _collateral: TokenAmount,
        _params: JoinParams,
    ) -> Result<()> {
        panic!("not implemented")
    }

    fn leave_subnet(_subnet: SubnetID, _from: Address) -> Result<()> {
        panic!("not implemented")
    }

    fn kill_subnet(_subnet: SubnetID, _from: Address) -> Result<()> {
        panic!("not implemented")
    }

    fn submit_checkpoint(_subnet: SubnetID, _from: Address, _ch: Checkpoint) -> Result<()> {
        panic!("not implemented")
    }

    fn list_child_subnets(_subnet: SubnetID) -> Result<HashMap<SubnetID, SubnetInfo>> {
        panic!("not implemented")
    }
}
