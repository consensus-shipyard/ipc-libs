// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::config::{ReloadableConfig, Subnet};
use crate::lotus::client::LotusJsonRPCClient;
use crate::manager::checkpoint::manager::bottomup::BottomUpCheckpointManager;
use crate::manager::checkpoint::manager::topdown::TopDownCheckpointManager;
use crate::manager::checkpoint::manager::CheckpointManager;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ipc_sdk::subnet_id::SubnetID;
use std::collections::HashMap;
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
                    // TODO: somehow adding the below lines causes: higher-ranked lifetime error
                    // TODO: comment off for now first.
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

fn handle_response(response: anyhow::Result<()>) {
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
                s.id.clone(),
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

    // A loop that drives stream to the end
    for manager in managers {
        let response = submit_next_epoch(manager).await;
        handle_response(response);
    }

    sleep_or_continue(start_time).await;

    Ok(())
}

async fn sleep_or_continue(start_time: Instant) {
    let elapsed = start_time.elapsed().as_secs();
    if elapsed < TASKS_PROCESS_THRESHOLD_SEC {
        sleep(Duration::from_secs(TASKS_PROCESS_THRESHOLD_SEC - elapsed)).await
    }
}

/// Attempts to submit checkpoint to the next `submittable` epoch. If the return value is Some(ChainEpoch).
/// it means the checkpoint is submitted to the target epoch. If returns None, it means there are no
/// epoch to be submitted.
async fn submit_next_epoch(manager: &impl CheckpointManager) -> Result<()> {
    let validators = manager.obtain_validators().await?;
    let period = manager.checkpoint_period();

    for validator in validators {
        log::debug!("submit checkpoint for validator: {validator:?}");

        while let Some(next_epoch) = manager.next_submission_epoch(&validator).await? {
            log::info!("next epoch to execute {next_epoch:} for validator {validator:}");

            let previous_epoch = next_epoch - period;
            manager
                .submit_checkpoint(next_epoch, previous_epoch, &validator)
                .await?;

            log::info!("checkpoint at epoch {next_epoch:} submitted for validator {validator:}");
        }
    }

    Ok(())
}
