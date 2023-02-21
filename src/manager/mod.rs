mod lotus_manager;

use std::collections::HashMap;

///! IPC node-specific traits.
use anyhow::Result;
use async_trait::async_trait;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_gateway::Checkpoint;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConstructParams, JoinParams, Status};
use serde::{Deserialize, Serialize};

/// Trait to interact with a subnet and handle its lifecycle.
#[async_trait]
pub trait SubnetManager {
    /// Deploys a new subnet actor on the `parent` subnet given as an input and with the
    /// configuration passed in `ConstructParams`.
    /// The result of the function is the ID address for the subnet actor from which the final
    /// subet ID can be inferred.
    fn create_subnet(parent: SubnetID, from: Address, params: ConstructParams) -> Result<Address>;

    /// Performs the call to join a subnet from a wallet address and staking an amount
    /// of collateral. This function, as well as all of the ones on this trait, can infer
    /// the specific subnet and actors on which to perform the relevant calls from the
    /// SubnetID given as an argument.
    fn join_subnet(
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        params: JoinParams,
    ) -> Result<()>;

    /// Sends a request to leave a subnet from a wallet address.
    fn leave_subnet(subnet: SubnetID, from: Address) -> Result<()>;

    /// Sends a signal to kill a subnet
    fn kill_subnet(subnet: SubnetID, from: Address) -> Result<()>;

    /// Submits a checkpoint for a subnet from a wallet address.
    fn submit_checkpoint(subnet: SubnetID, from: Address, ch: Checkpoint) -> Result<()>;

    /// Lists all the registered children for a subnet.
    fn list_child_subnets(subnet: SubnetID) -> Result<HashMap<SubnetID, SubnetInfo>>;
}

/// SubnetInfo is an auxiliary struct that collects
/// relevant information about the state of a subnet
#[derive(Debug, Serialize, Deserialize)]
pub struct SubnetInfo {
    /// Name of the subnet.
    pub name: String,
    /// Collateral staked in the subnet.
    pub collateral: TokenAmount,
    /// Circulating supply available in the subnet.
    pub circ_supply: TokenAmount,
    /// State of the Subnet (Initialized, Active, Killed)
    pub status: Status,
}
