use crate::config::{ReloadableConfig, Subnet};
use crate::manager::checkpoint::manager::CheckpointManager;
use crate::manager::checkpoint::submit::SubmissionStrategy;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::select;
use tokio_graceful_shutdown::{IntoSubsystem, SubsystemHandle};

const EXECUTION_BATCH_SIZE: usize = 20;

pub struct CheckpointSubsystem {
    driver: CheckpointDriver,
}

#[async_trait]
impl IntoSubsystem<anyhow::Error> for CheckpointSubsystem {
    async fn run(self, subsys: SubsystemHandle) -> anyhow::Result<()> {
        select! {
            // TODO: somehow adding the below lines causes: higher-ranked lifetime error
            // TODO: comment off for now first.
            // _ = tokio::spawn(async move {
            //     self.driver.run().await;
            // }) => Ok::<(), anyhow::Error>(()),
            _ = subsys.on_shutdown_requested() => {
                log::info!("Shutting down checkpointing subsystem");
                Ok(())
            }
        }
    }
}

struct CheckpointDriver {
    /// The subsystem uses a `ReloadableConfig` to ensure that, at all, times, the subnets under
    /// management are those in the latest version of the config.
    config: Arc<ReloadableConfig>,
}

impl CheckpointDriver {
    pub async fn run(&self) -> anyhow::Result<()> {
        // Each event in this channel is notification of a new config.
        let mut config_chan = self.config.new_subscriber();

        loop {
            // Load the latest config.
            let config = self.config.get_config();
            let managers = setup_managers_from_config(&config.subnets)?;

            // Each manager might have to handle multiple validators, we group them into
            // (Manager, Validator) tuple pair, so that we can fire the submission in batches
            let groups = break_into_groups(managers).await?;

            loop {
                let mut stream = tokio_stream::iter(&groups)
                    .map(|(policy, validator)| async move {
                        policy.try_submit_next_epoch(validator).await.map_err(|e| {
                            log::error!("manager: {:} failed with error: {:}", policy.id(), e);
                            e
                        })
                    })
                    .buffer_unordered(EXECUTION_BATCH_SIZE);

                loop {
                    select! {
                        r = stream.next() => match r {
                            Some(response) => handle_response(response),
                            None => break,
                        },
                        r = config_chan.recv() => {
                            log::info!("Config changed, reloading checkpointing subsystem");
                            match r {
                                Ok(_) => { break },
                                Err(_) => {
                                    return Err(anyhow!("Config channel unexpectedly closed, shutting down checkpointing subsystem"))
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl<P: CheckpointManager + Send + Sync> CheckpointManagerWrapper for P {
    /// Attempts to submit checkpoint to the next `submittable` epoch. If the return value is Some(ChainEpoch).
    /// it means the checkpoint is submitted to the target epoch. If returns None, it means there are no
    /// epoch to be submitted.
    async fn try_submit_next_epoch(&self, validator: &Address) -> Result<()> {
        let period = self.checkpoint_period();
        let target_subnet = self.child_subnet();

        log::debug!("submit checkpoint for validator: {validator:?}");

        while let Some(next_epoch) = self.next_submission_epoch(validator).await? {
            log::info!(
                        "next epoch to execute {next_epoch:} for validator {validator:} in subnet {target_subnet:}"
                    );

            let previous_epoch = next_epoch - period;

            let checkpoint = self.create_checkpoint(next_epoch, previous_epoch).await?;
            log::info!("next epoch to execute {next_epoch:} for validator {validator:} with checkpoint {checkpoint:?}");

            self.submission_strategy()
                .submit_checkpoint(target_subnet, next_epoch, validator, checkpoint)
                .await?;

            log::info!("checkpoint at epoch {next_epoch:} submitted for validator {validator:} in subnet {target_subnet:}");
        }

        Ok(())
    }

    async fn setup(&self) -> Result<Vec<Address>> {
        self.sync_checkpoint_period().await?;
        self.obtain_validators().await
    }

    fn id(&self) -> String {
        format!(
            "parent({:})-child({:})-type({:?})",
            self.parent_subnet(),
            self.child_subnet(),
            self.checkpoint_type()
        )
    }
}

/// A util trait to avoid Box<dyn> and associated type mess in CheckpointPolicy trait
#[async_trait]
trait CheckpointManagerWrapper: Send + Sync {
    /// Try submit the checkpoint for the validator in the checkpoint policy
    async fn try_submit_next_epoch(&self, validator: &Address) -> Result<()>;

    /// Setup the checkpoint policy
    async fn setup(&self) -> Result<Vec<Address>>;

    fn id(&self) -> String;
}

fn handle_response(response: anyhow::Result<()>) {
    if response.is_err() {
        // TODO: handle different actor error responses
    }
}

fn setup_managers_from_config(
    _subnets: &HashMap<SubnetID, Subnet>,
) -> Result<Vec<Box<dyn CheckpointManagerWrapper>>> {
    todo!()
    // log::debug!("We have {} subnets to manage", subnets_to_manage.len());
}

async fn break_into_groups(
    policies: Vec<Box<dyn CheckpointManagerWrapper>>,
) -> anyhow::Result<Vec<(Arc<Box<dyn CheckpointManagerWrapper>>, Address)>> {
    let mut pairs = vec![];
    for p in policies {
        let validators = p.setup().await?;

        let p = Arc::new(p);
        for validator in validators {
            pairs.push((p.clone(), validator));
        }
    }

    Ok(pairs)
}
