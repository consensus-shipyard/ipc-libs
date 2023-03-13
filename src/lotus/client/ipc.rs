use crate::jsonrpc::JsonRpcClient;
use crate::lotus::client::GATEWAY_ACTOR_ADDRESS;
use crate::lotus::client::{methods, LotusJsonRPCClient};
use crate::lotus::message::ipc::{
    IPCGetPrevCheckpointForChildResponse, IPCReadGatewayStateResponse,
    IPCReadSubnetActorStateResponse,
};
use crate::lotus::message::CIDMap;
use crate::lotus::LotusIPCClient;
use anyhow::anyhow;
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use serde_json::json;

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> LotusIPCClient for LotusJsonRPCClient<T> {
    async fn get_prev_checkpoint_for_child(
        &self,
        child_subnet_id: SubnetID,
    ) -> anyhow::Result<IPCGetPrevCheckpointForChildResponse> {
        let parent = match child_subnet_id.parent() {
            None => return Err(anyhow!("The child_subnet_id must be a valid child subnet")),
            Some(parent) => parent,
        };
        let subnet_actor = child_subnet_id.subnet_actor().to_string();
        let params =
            json!([GATEWAY_ACTOR_ADDRESS, {"Parent": parent.to_string(), "Actor": subnet_actor}]);

        let r = self
            .client
            .request::<IPCGetPrevCheckpointForChildResponse>(
                methods::IPC_GET_PREV_CHECKPOINT_FOR_CHILD,
                params,
            )
            .await?;
        Ok(r)
    }

    async fn get_checkpoint_template(&self, epoch: ChainEpoch) -> anyhow::Result<Checkpoint> {
        let r = self
            .client
            .request::<Checkpoint>(
                methods::IPC_GET_CHECKPOINT_TEMPLATE,
                json!([GATEWAY_ACTOR_ADDRESS, epoch]),
            )
            .await?;
        Ok(r)
    }

    async fn read_gateway_state(
        &self,
        tip_set: Cid,
    ) -> anyhow::Result<IPCReadGatewayStateResponse> {
        let params = json!([GATEWAY_ACTOR_ADDRESS, [CIDMap::from(tip_set)]]);
        let r = self
            .client
            .request::<IPCReadGatewayStateResponse>(methods::IPC_READ_GATEWAY_STATE, params)
            .await?;
        Ok(r)
    }

    async fn read_subnet_actor_state(
        &self,
        tip_set: Cid,
    ) -> anyhow::Result<IPCReadSubnetActorStateResponse> {
        let params = json!([GATEWAY_ACTOR_ADDRESS, [CIDMap::from(tip_set)]]);
        let r = self
            .client
            .request::<IPCReadSubnetActorStateResponse>(
                methods::IPC_READ_SUBNET_ACTOR_STATE,
                params,
            )
            .await?;
        Ok(r)
    }

    async fn fund(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _amount: TokenAmount,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn release(&self, _subnet: SubnetID, _from: Address) -> anyhow::Result<()> {
        todo!()
    }

    async fn propagate(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _postbox_cid: Cid,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn whitelist_propagator(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _postbox_cid: Cid,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
