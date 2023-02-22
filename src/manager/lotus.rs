use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use fil_actors_runtime::cbor;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConstructParams, JoinParams};
use crate::jsonrpc::JsonRpcClient;
use crate::lotus::LotusJsonRPCClient;

use super::subnet::{SubnetInfo, SubnetManager};

pub struct LotusSubnetManager<T: JsonRpcClient> {
    lotus_client: LotusJsonRPCClient<T>
}

#[async_trait]
impl <T: JsonRpcClient> SubnetManager for LotusSubnetManager<T> {
    fn create_subnet(
        _parent: SubnetID,
        from: Address,
        params: ConstructParams,
    ) -> Result<Address> {
        let construct_params = cbor::serialize(&params, "create subnet actor")?;

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
