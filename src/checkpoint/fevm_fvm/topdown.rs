use crate::config::Subnet;
use fvm_shared::clock::ChainEpoch;

/// Top down checkpoint manager. It reads the state of parent subnet, FVM, and commits to child subnet,
/// FEVM.
#[warn(dead_code)]
pub struct TopDownCheckpointManager {
    parent: Subnet,
    child: Subnet,
    checkpoint_period: ChainEpoch,
    fvm_top_down_manager: u64,
    fevm_top_down_manager: u64,
}
