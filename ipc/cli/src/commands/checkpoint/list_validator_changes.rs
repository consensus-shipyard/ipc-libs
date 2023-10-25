// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! List validator change set cli command

use std::fmt::Debug;
use std::str::FromStr;

use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;
use ipc_sdk::subnet_id::SubnetID;

use crate::commands::get_ipc_provider;
use crate::{CommandLineHandler, GlobalArguments};

/// The command to list checkpoints committed in a subnet actor.
pub(crate) struct ListValidatorChanges;

#[async_trait]
impl CommandLineHandler for ListValidatorChanges {
    type Arguments = ListValidatorChangesArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("list validator changes with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;

        for h in arguments.from_epoch..=arguments.to_epoch {
            let changes = provider.get_validator_changeset(&subnet, h).await?;
            log::info!("changes at height: {h} are: {:?}", changes.value);
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "List bottom-up checkpoints")]
pub(crate) struct ListValidatorChangesArgs {
    #[arg(long, short, help = "The subnet id of the checkpointing subnet")]
    pub subnet: String,
    #[arg(long, short, help = "Include checkpoints from this epoch")]
    pub from_epoch: ChainEpoch,
    #[arg(long, short, help = "Include checkpoints up to this epoch")]
    pub to_epoch: ChainEpoch,
}
