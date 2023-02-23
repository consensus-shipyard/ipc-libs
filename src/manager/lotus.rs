use std::collections::HashMap;
use std::str::FromStr;

use crate::jsonrpc::JsonRpcClient;
use crate::lotus::{LotusClient, LotusJsonRPCClient, MpoolPushMessage, NetworkVersion};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use fil_actors_runtime::{builtin::singletons::INIT_ACTOR_ADDR, cbor};
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConstructParams, JoinParams, types::MANIFEST_ID};
use crate::manager::params::{ExecParams, INIT_EXEC_METHOD_NUM};
use super::subnet::{SubnetInfo, SubnetManager};

pub struct LotusSubnetManager<T: JsonRpcClient> {
    lotus_client: LotusJsonRPCClient<T>,
}

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> SubnetManager for LotusSubnetManager<T> {
    async fn create_subnet(&self, from: Address, params: ConstructParams) -> Result<Address> {
        let network_name = self.lotus_client.state_network_name().await?;
        if params.parent.to_string() != network_name {
            return Err(anyhow!("parent network name not match"));
        }

        let actor_code_cid = self.get_subnet_actor_code_cid().await?;
        let constructor_params = cbor::serialize(&params, "create subnet actor")?;

        let exec_params = ExecParams {
            code_cid: actor_code_cid,
            constructor_params,
        };
        log::debug!("create subnet for init actor with params: {exec_params:?}");
        let init_params = cbor::serialize(&exec_params, "init subnet actor params")?;
        let message = MpoolPushMessage::new(
            INIT_ACTOR_ADDR,
            from,
            INIT_EXEC_METHOD_NUM,
            init_params.to_vec(),
        );

        let mem_push_response = self.lotus_client.mpool_push_message(message).await?;
        let message_cid = mem_push_response.cid()?;
        let nonce = mem_push_response.nonce;
        log::debug!(
            "create subnet message published with cid: {message_cid:?} and nonce: {nonce:?}"
        );

        let state_wait_response = self.lotus_client.state_wait_msg(message_cid, nonce).await?;
        let address_raw = state_wait_response.receipt.result;
        log::info!("created subnet at address: {address_raw:}");

        Ok(Address::from_str(&address_raw)?)
    }

    async fn join_subnet(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _collateral: TokenAmount,
        _params: JoinParams,
    ) -> Result<()> {
        panic!("not implemented")
    }

    async fn leave_subnet(&self, _subnet: SubnetID, _from: Address) -> Result<()> {
        panic!("not implemented")
    }

    async fn kill_subnet(&self, _subnet: SubnetID, _from: Address) -> Result<()> {
        panic!("not implemented")
    }

    async fn submit_checkpoint(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _ch: Checkpoint,
    ) -> Result<()> {
        panic!("not implemented")
    }

    async fn list_child_subnets(&self, _subnet: SubnetID) -> Result<HashMap<SubnetID, SubnetInfo>> {
        panic!("not implemented")
    }
}

impl<T: JsonRpcClient + Send + Sync> LotusSubnetManager<T> {
    /// Obtain the actor code cid of `ipc_subnet_actor` only, since this is the
    /// code cid we are interested in.
    async fn get_subnet_actor_code_cid(&self) -> Result<Cid> {
        let network_version = self.lotus_client.state_network_version(vec![]).await?;

        let mut cid_map = self
            .lotus_client
            .state_actor_code_cids(network_version)
            .await?;

        cid_map
            .remove(MANIFEST_ID)
            .ok_or(anyhow!("actor cid not found"))
    }
}
