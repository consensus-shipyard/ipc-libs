use crate::manager::checkpoint::CheckpointManager;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::BottomUpCheckpoint;
use ipc_sdk::subnet_id::SubnetID;

pub struct BottomUpCheckpointManager;

#[async_trait]
impl CheckpointManager for BottomUpCheckpointManager {
    type Checkpoint = BottomUpCheckpoint;

    fn parent_subnet(&self) -> &SubnetID {
        todo!()
    }

    fn child_subnet(&self) -> &SubnetID {
        todo!()
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        todo!()
    }

    async fn sync_checkpoint_period(&self) -> anyhow::Result<()> {
        todo!()
    }

    async fn obtain_validators(&self) -> anyhow::Result<Vec<Address>> {
        todo!()
    }

    async fn submit_checkpoint(
        &self,
        _epoch: ChainEpoch,
        _previous_epoch: ChainEpoch,
        _validator: &Address,
    ) -> anyhow::Result<Self::Checkpoint> {
        todo!()
    }

    async fn next_submission_epoch(
        &self,
        _validator: &Address,
    ) -> anyhow::Result<Option<ChainEpoch>> {
        todo!()
    }
}
