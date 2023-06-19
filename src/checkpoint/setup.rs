use crate::checkpoint::bottomup::BottomUpCheckpointManager as FVMBottomUpCheckpointManager;
use crate::checkpoint::fevm::BottomUpCheckpointManager as FEVMBottomUpCheckpointManager;
use crate::checkpoint::fevm::TopdownCheckpointManager as FEVMTopdownCheckpointManager;
use crate::checkpoint::topdown::TopDownCheckpointManager as FVMTopDownCheckpointManager;
use crate::checkpoint::CheckpointManager;
use crate::config::subnet::NetworkType;
use crate::config::Subnet;
use crate::lotus::client::LotusJsonRPCClient;
use crate::manager::EthSubnetManager;
use anyhow::anyhow;
use ipc_identity::Wallet;
use ipc_sdk::subnet_id::SubnetID;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

async fn parent_fvm_child_fvm(
    parent: &Subnet,
    child: &Subnet,
    wallet_store: Arc<RwLock<Wallet>>,
) -> anyhow::Result<Vec<Box<dyn CheckpointManager>>> {
    if parent.network_type() != NetworkType::Fvm || child.network_type() != NetworkType::Fvm {
        return Err(anyhow!("parent not fvm or child not fvm"));
    }

    let mut managers = vec![];
    let m: Box<dyn CheckpointManager> = Box::new(
        FVMBottomUpCheckpointManager::new(
            LotusJsonRPCClient::from_subnet_with_wallet_store(parent, wallet_store.clone()),
            parent.clone(),
            LotusJsonRPCClient::from_subnet_with_wallet_store(child, wallet_store.clone()),
            child.clone(),
        )
        .await?,
    );

    managers.push(m);

    let m: Box<dyn CheckpointManager> = Box::new(
        FVMTopDownCheckpointManager::new(
            LotusJsonRPCClient::from_subnet_with_wallet_store(parent, wallet_store.clone()),
            parent.clone(),
            LotusJsonRPCClient::from_subnet_with_wallet_store(child, wallet_store.clone()),
            child.clone(),
        )
        .await?,
    );

    managers.push(m);

    Ok(managers)
}

async fn parent_fevm_child_fevm(
    parent: &Subnet,
    child: &Subnet,
    fvm_wallet_store: Arc<RwLock<Wallet>>,
) -> anyhow::Result<Vec<Box<dyn CheckpointManager>>> {
    if parent.network_type() != NetworkType::Fevm || child.network_type() != NetworkType::Fevm {
        return Err(anyhow!("parent not fevm or child not fevm"));
    }

    let mut managers = vec![];
    let m: Box<dyn CheckpointManager> = Box::new(
        FEVMBottomUpCheckpointManager::new(
            parent.clone(),
            EthSubnetManager::from_subnet(parent)?,
            child.clone(),
            EthSubnetManager::from_subnet(child)?,
            LotusJsonRPCClient::from_subnet_with_wallet_store(child, fvm_wallet_store),
        )
        .await?,
    );

    managers.push(m);

    let m: Box<dyn CheckpointManager> = Box::new(
        FEVMTopdownCheckpointManager::new(
            parent.clone(),
            EthSubnetManager::from_subnet(parent)?,
            child.clone(),
            EthSubnetManager::from_subnet(child)?,
        )
        .await?,
    );

    managers.push(m);

    Ok(managers)
}

pub async fn setup_manager_from_subnet(
    subnets: &HashMap<SubnetID, Subnet>,
    s: &Subnet,
    fvm_wallet_store: Arc<RwLock<Wallet>>,
) -> anyhow::Result<Vec<Box<dyn CheckpointManager>>> {
    let parent = if let Some(p) = s.id.parent() && subnets.contains_key(&p) {
        subnets.get(&p).unwrap()
    } else {
        log::info!("subnet has no parent configured: {:}, not managing checkpoints", s.id);
        return Ok(vec![]);
    };

    match (parent.network_type(), s.network_type()) {
        (NetworkType::Fvm, NetworkType::Fvm) => {
            parent_fvm_child_fvm(parent, s, fvm_wallet_store).await
        }
        (NetworkType::Fvm, NetworkType::Fevm) => {
            unimplemented!()
        }
        (NetworkType::Fevm, NetworkType::Fvm) => {
            unimplemented!()
        }
        (NetworkType::Fevm, NetworkType::Fevm) => {
            parent_fevm_child_fevm(parent, s, fvm_wallet_store).await
        }
    }
}

pub async fn setup_managers_from_config(
    subnets: &HashMap<SubnetID, Subnet>,
    fvm_wallet_store: Arc<RwLock<Wallet>>,
) -> anyhow::Result<Vec<Box<dyn CheckpointManager>>> {
    let mut managers = vec![];

    for s in subnets.values() {
        log::info!("config checkpoint manager for subnet: {:}", s.id);

        // TODO: once we have added EVM wallet store, we should switch to the below approach.
        // // We filter for subnets that have at least one account and for which the parent subnet
        // // is also in the configuration.
        // if s.accounts().is_empty() {
        //     log::info!("no accounts in subnet: {:}, not managing checkpoints", s.id);
        //     continue;
        // }

        let subnet_managers =
            setup_manager_from_subnet(subnets, s, fvm_wallet_store.clone()).await?;
        managers.extend(subnet_managers);
    }

    log::info!(
        "we are managing checkpoints for {:} number of subnets",
        managers.len()
    );

    Ok(managers)
}
