//! Checkpoint manager with FEVM as parent and FVM as child

use crate::checkpoint::fevm::BottomUpCheckpointManager as FEVMBottomUpCheckpointManager;
use crate::checkpoint::fevm::TopdownCheckpointManager as FEVMTopdownCheckpointManager;
use crate::checkpoint::fvm::bottomup::BottomUpCheckpointManager as FVMBottomUpCheckpointManager;
use crate::checkpoint::fvm::topdown::TopDownCheckpointManager as FVMTopdownCheckpointManager;
use crate::config::Subnet;
use crate::manager::EthManager;
use fvm_shared::clock::ChainEpoch;

/// Bottom up checkpoint manager. It reads the state of child subnet, FVM, and commits to parent subnet,
/// FEVM.
pub struct BottomUpCheckpointManager<ParentManager, ChildManager> {
    parent: Subnet,
    child: Subnet,
    checkpoint_period: ChainEpoch,
    parent_fvm_bu_manager: ParentManager,
    child_fevm_bu_manager: ChildManager,
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
