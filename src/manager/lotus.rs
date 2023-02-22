use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use fil_actors_runtime::{cbor, builtin::singletons::INIT_ACTOR_ADDR};
use fvm_ipld_encoding::RawBytes;
use fvm_shared::{address::Address, econ::TokenAmount, MethodNum};
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConstructParams, JoinParams};
use fvm_ipld_encoding::tuple::{Serialize_tuple, Deserialize_tuple};
use crate::jsonrpc::JsonRpcClient;
use crate::lotus::{LotusClient, LotusJsonRPCClient, MpoolPushMessage, NetworkVersion};

use super::subnet::{SubnetInfo, SubnetManager};

/// To be imported from gateway actor once merged.
const IPC_SUBNET_ACTOR_MANIFEST: &str = "ipc_subnet_actor";

/// Init actor Exec Params, see https://github.com/filecoin-project/builtin-actors/blob/master/actors/init/src/types.rs#L17
#[derive(Serialize_tuple, Deserialize_tuple)]
struct ExecParams {
    code_cid: Cid,
    constructor_params: RawBytes,
}
/// Init actor exec method number, see https://github.com/filecoin-project/builtin-actors/blob/fb759f87fcd5de0a98cb61966cd27f680df83364/actors/init/src/lib.rs#L32
const INIT_EXEC_METHOD_NUM: MethodNum = 2;

pub struct LotusSubnetManager<T: JsonRpcClient> {
    lotus_client: LotusJsonRPCClient<T>
}

#[async_trait]
impl <T: JsonRpcClient + Send + Sync> SubnetManager for LotusSubnetManager<T> {
    async fn create_subnet(
        &self,
        from: Address,
        params: ConstructParams,
    ) -> Result<Address> {
        let network_name = self.lotus_client.state_network_name().await?;
        if params.parent.to_string() != network_name {
            return Err(anyhow!("parent network name not match"));
        }

        let construct_params = cbor::serialize(&params, "create subnet actor")?;
        let actor_code_cid = self.get_subnet_actor_code_cid().await?;

        let init_params = cbor::serialize(
            &ExecParams { code_cid: actor_code_cid, constructor_params: construct_params },
            "init subnet actor params"
        )?;
        let message = MpoolPushMessage::new(INIT_ACTOR_ADDR, from, INIT_EXEC_METHOD_NUM, init_params.to_vec());

        let mem_push_response = self.lotus_client.mpool_push_message(message).await?;
        let message_cid = mem_push_response.get_root_cid().ok_or(anyhow!("cid not returned in mpool push"))?;
        let nonce = mem_push_response.nonce;

        let state_wait_response = self.lotus_client.state_wait_msg(message_cid, nonce).await?;
        let address_raw = state_wait_response.receipt.r#return;

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

    async fn submit_checkpoint(&self, _subnet: SubnetID, _from: Address, _ch: Checkpoint) -> Result<()> {
        panic!("not implemented")
    }

    async fn list_child_subnets(&self, _subnet: SubnetID) -> Result<HashMap<SubnetID, SubnetInfo>> {
        panic!("not implemented")
    }
}

impl <T: JsonRpcClient + Send + Sync> LotusSubnetManager<T> {
    async fn get_subnet_actor_code_cid(&self) -> Result<Cid> {
        let network_version = self.lotus_client.state_network_version(vec![]).await?;
        self.state_actor_code_cids(network_version).await
    }

    async fn state_actor_code_cids(&self, network_version: NetworkVersion) -> Result<Cid> {
        let mut cid_map = self.lotus_client.state_actor_code_cids(network_version).await?;
        cid_map.remove(IPC_SUBNET_ACTOR_MANIFEST).ok_or(anyhow!("actor cid not found"))
    }
}
