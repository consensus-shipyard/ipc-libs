use crate::checkpoint::CheckpointManager;
use crate::config::Subnet;
use async_trait::async_trait;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use std::fmt::{Display, Formatter};

struct BottomUpCheckpointManager {
    parent_subnet: Subnet,
    child_subnet: Subnet,
    checkpoint_period: ChainEpoch,
}

impl Display for BottomUpCheckpointManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fevm bottom-up, parent: {:}, child: {:}",
            self.parent_subnet.id, self.child_subnet.id
        )
    }
}

#[async_trait]
impl CheckpointManager for BottomUpCheckpointManager {
    fn parent_subnet(&self) -> &Subnet {
        &self.parent_subnet
    }

    fn child_subnet(&self) -> &Subnet {
        &self.child_subnet
    }

    fn checkpoint_period(&self) -> ChainEpoch {
        self.checkpoint_period
    }

    async fn child_validators(&self) -> anyhow::Result<Vec<Address>> {
        // Current solidity contract needs to support batch query
        todo!()
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn submit_checkpoint(
        &self,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        todo!()
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        todo!()
    }
}
