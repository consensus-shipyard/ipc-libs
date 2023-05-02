// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::config::{ReloadableConfig, Subnet};
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::LotusClient;
use crate::manager::checkpoint::manager::bottomup::BottomUpCheckpointManager;
use crate::manager::checkpoint::manager::topdown::TopDownCheckpointManager;
use crate::manager::checkpoint::manager::CheckpointManager;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use futures_util::future::join_all;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::select;
use tokio::time::sleep;
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};

const TASKS_PROCESS_THRESHOLD_SEC: u64 = 15;

pub struct CheckpointSubsystem {
    /// The subsystem uses a `ReloadableConfig` to ensure that, at all, times, the subnets under
    /// management are those in the latest version of the config.
    config: Arc<ReloadableConfig>,
}

#[async_trait]
impl IntoSubsystem<anyhow::Error> for CheckpointSubsystem {
    async fn run(self, subsys: SubsystemHandle) -> anyhow::Result<()> {
        // Each event in this channel is notification of a new config.
        let mut config_chan = self.config.new_subscriber();

        loop {
            // Load the latest config.
            let config = self.config.get_config();
            let (top_down_managers, bottom_up_managers) =
                setup_managers_from_config(&config.subnets).await?;

            loop {
                select! {
                    _ = process_managers(&top_down_managers) => {},
                    _ = process_managers(&bottom_up_managers) => {},
                    r = config_chan.recv() => {
                        log::info!("Config changed, reloading checkpointing subsystem");
                        match r {
                            // Config updated, return to caller
                            Ok(_) => { break; },
                            Err(_) => {
                                return Err(anyhow!("Config channel unexpectedly closed, shutting down checkpointing subsystem"))
                            },
                        }
                    }
                    _ = subsys.on_shutdown_requested() => {
                        log::info!("Shutting down checkpointing subsystem");
                        return Ok(());
                    }
                }
            }
        }
    }
}

fn handle_err_response(response: anyhow::Result<()>) {
    if response.is_err() {
        // TODO: handle different actor error responses
    }
}

async fn setup_managers_from_config(
    subnets: &HashMap<SubnetID, Subnet>,
) -> Result<(
    Vec<TopDownCheckpointManager>,
    Vec<BottomUpCheckpointManager>,
)> {
    let mut bottom_up_managers = vec![];
    let top_down_managers = vec![];

    for s in subnets.values() {
        log::info!("config checkpoint manager for subnet: {:}", s.id);

        // We filter for subnets that have at least one account and for which the parent subnet
        // is also in the configuration.
        if s.accounts.is_empty() {
            log::info!("no accounts in subnet: {:}, not managing checkpoints", s.id);
            continue;
        }

        let parent = if let Some(p) = s.id.parent() && subnets.contains_key(&p) {
            subnets.get(&p).unwrap()
        } else {
            log::info!("subnet has no parent configured: {:}, not managing checkpoints", s.id);
            continue
        };

        bottom_up_managers.push(
            BottomUpCheckpointManager::new(
                LotusJsonRPCClient::from_subnet(parent),
                parent.id.clone(),
                LotusJsonRPCClient::from_subnet(s),
                s.clone(),
            )
            .await?,
        );

        // TODO: to update top down in another PR
    }

    log::info!(
        "we are managing checkpoints for {:} number of bottom up subnets",
        bottom_up_managers.len()
    );
    log::info!(
        "we are managing checkpoints for {:} number of top down subnets",
        top_down_managers.len()
    );

    Ok((top_down_managers, bottom_up_managers))
}

async fn process_managers<T: CheckpointManager>(managers: &[T]) -> anyhow::Result<()> {
    // Tracks the start time of the processing, will use this to determine should sleep
    let start_time = Instant::now();

    let futures = managers
        .iter()
        .map(|manager| async {
            let response = submit_till_current_epoch(manager).await;
            handle_err_response(response);
        })
        .collect::<Vec<_>>();

    join_all(futures).await;

    sleep_or_continue(start_time).await;

    Ok(())
}

async fn sleep_or_continue(start_time: Instant) {
    let elapsed = start_time.elapsed().as_secs();
    if elapsed < TASKS_PROCESS_THRESHOLD_SEC {
        sleep(Duration::from_secs(TASKS_PROCESS_THRESHOLD_SEC - elapsed)).await
    }
}

/// Attempts to submit checkpoints from the last executed epoch all the way to the current epoch for
/// all the validators in the provided manager.
async fn submit_till_current_epoch(manager: &impl CheckpointManager) -> Result<()> {
    // we might have to obtain the list of validators as some validators might leave the subnet
    // we can improve the performance by caching if this slows down the process significantly.
    let validators = obtain_validators(manager).await?;
    let period = manager.checkpoint_period();

    let last_executed_epoch = manager.last_executed_epoch().await?;
    let current_epoch = manager.current_epoch().await?;

    log::debug!(
        "latest epoch {:?}, last executed epoch: {:?} for checkpointing: {:}",
        current_epoch,
        last_executed_epoch,
        manager,
    );

    let mut next_epoch = last_executed_epoch + period;
    while next_epoch < current_epoch {
        // now we process each validator
        for validator in &validators {
            log::debug!("submit checkpoint for validator: {validator:?} in manager: {manager:}");

            if manager.has_submitted_epoch(validator, next_epoch).await? {
                log::debug!(
                    "next submission epoch {next_epoch:?} already voted for validator: {validator:?} in manager: {manager:}"
                );
                continue;
            }

            log::debug!(
                "next submission epoch {next_epoch:} not voted for validator: {validator:} in manager: {manager:}, should vote"
            );

            manager
                .submit_checkpoint(next_epoch, next_epoch - period, validator)
                .await?;

            log::info!("checkpoint at epoch {next_epoch:} submitted for validator {validator:} in manager: {manager:}");
        }

        // increment next epoch
        next_epoch += period;
    }

    log::info!("process checkpoint from epoch: {last_executed_epoch:} to {current_epoch:} in manager: {manager:}");

    Ok(())
}

/// Obtain the validators in the subnet from the parent subnet of the manager
async fn obtain_validators(manager: &impl CheckpointManager) -> anyhow::Result<Vec<Address>> {
    let parent_client = manager.parent_client();
    let parent_head = parent_client.chain_head().await?;

    // A key assumption we make now is that each block has exactly one tip set. We panic
    // if this is not the case as it violates our assumption.
    // TODO: update this logic once the assumption changes (i.e., mainnet)
    assert_eq!(parent_head.cids.len(), 1);

    let cid_map = parent_head.cids.first().unwrap().clone();
    let parent_tip_set = Cid::try_from(cid_map)?;
    let child_subnet = manager.child_subnet();

    let subnet_actor_state = parent_client
        .ipc_read_subnet_actor_state(&child_subnet.id, parent_tip_set)
        .await?;

    match subnet_actor_state.validator_set.validators {
        None => Ok(vec![]),
        Some(validators) => {
            let mut vs = vec![];
            for v in validators {
                let addr = Address::from_str(&v.addr)?;
                if child_subnet.accounts.contains(&addr) {
                    vs.push(addr);
                }
            }
            Ok(vs)
        }
    }
}
