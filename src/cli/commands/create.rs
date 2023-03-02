// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet cli command handler.

use async_trait::async_trait;
use clap::Args;
use fvm_shared::clock::ChainEpoch;
use std::fmt::Debug;

use crate::cli::CommandLineHandler;
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::{CreateSubnetParams, CreateSubnetResponse};

/// The command to create a new subnet actor.
pub(crate) struct CreateSubnet;

#[async_trait]
impl CommandLineHandler for CreateSubnet {
    type Arguments = CreateSubnetArgs;

    async fn handle(arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("launching json rpc server with args: {:?}", arguments);

        let params = CreateSubnetParams {
            parent: arguments.parent.clone(),
            name: arguments.name.clone(),
            min_validator_stake: arguments.min_validator_stake,
            min_validators: arguments.min_validators,
            finality_threshold: arguments.finality_threshold,
            check_period: arguments.check_period,
        };

        let url = arguments.ipc_agent_url.parse()?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let address = json_rpc_client
            .request::<CreateSubnetResponse>(
                json_rpc_methods::CREATE_SUBNET,
                serde_json::to_value(params)?,
            )
            .await?
            .address;

        log::info!("created subent actor with address: {address:}");

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Create a new subnet actor")]
pub(crate) struct CreateSubnetArgs {
    #[arg(help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: String,
    #[arg(help = "The parent subnet to create the new actor in")]
    pub parent: String,
    #[arg(help = "The name of the subnet")]
    pub name: String,
    #[arg(help = "The minimal validator stake amount")]
    pub min_validator_stake: u64,
    #[arg(help = "The minimal number of validators")]
    pub min_validators: u64,
    #[arg(help = "The finality threshold for MIR")]
    pub finality_threshold: ChainEpoch,
    #[arg(help = "The checkpoint period")]
    pub check_period: ChainEpoch,
}
