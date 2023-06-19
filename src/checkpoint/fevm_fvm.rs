//! Checkpoint manager with FEVM as parent and FVM as child

use std::fmt::{Display, Formatter};
use fvm_shared::address::Address;
use crate::checkpoint::fevm::BottomUpCheckpointManager as FEVMBottomUpCheckpointManager;
use crate::checkpoint::fevm::TopdownCheckpointManager as FEVMTopdownCheckpointManager;
use crate::checkpoint::fvm::bottomup::BottomUpCheckpointManager as FVMBottomUpCheckpointManager;
use crate::checkpoint::fvm::topdown::TopDownCheckpointManager as FVMTopdownCheckpointManager;
use crate::config::Subnet;
use crate::manager::EthManager;
use fvm_shared::clock::ChainEpoch;
use crate::checkpoint::{CheckpointManager};
use crate::lotus::LotusClient;

/// Bottom up checkpoint manager. It reads the state of child subnet, FVM, and commits to parent subnet,
/// FEVM.
pub struct BottomUpCheckpointManager<ParentManager, ChildManager> {
    parent: Subnet,
    child: Subnet,
    checkpoint_period: ChainEpoch,
    parent_fevm_manager: ParentManager,
    child_fvm_manager: ChildManager,
}

impl<P, M> Display for BottomUpCheckpointManager<P, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fvm to fevm bottom-up, parent: {:}, child: {:}",
            self.parent_subnet.id, self.child_subnet.id
        )
    }
}

impl <P: EthManager + Send + Sync, C: LotusClient + Send + Sync> CheckpointManager for BottomUpCheckpointManager<P, C> {
    fn parent_subnet(&self) -> &Subnet { &self.parent }

    fn child_subnet(&self) -> &Subnet { &self.child }

    fn checkpoint_period(&self) -> ChainEpoch { self.checkpoint_period }

    async fn child_validators(&self) -> anyhow::Result<Vec<Address>> {
        self.parent_fevm_manager.validators(&self.child_subnet.id).await
    }

    async fn last_executed_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.parent_fevm_manager
            .gateway_last_voting_executed_epoch()
            .await
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        self.child_fvm_manager.current_epoch().await
    }

    async fn submit_checkpoint(&self, epoch: ChainEpoch, validator: &Address) -> anyhow::Result<()> {
        todo!()
    }

    async fn should_submit_in_epoch(
        &self,
        validator: &Address,
        epoch: ChainEpoch,
    ) -> anyhow::Result<bool> {
        self.parent_fevm_manager
            .has_voted_in_subnet(&self.child_subnet.id, epoch, validator)
            .await
    }

    async fn presubmission_check(&self) -> anyhow::Result<bool> {
        Ok(true)
    }
}

/// Top down checkpoint manager. It reads the state of parent subnet, FVM, and commits to child subnet,
/// FEVM.
pub struct TopDownCheckpointManager {
    parent: Subnet,
    child: Subnet,
    checkpoint_period: ChainEpoch,
    fvm_top_down_manager: u64,
    fevm_top_down_manager: u64,
}
