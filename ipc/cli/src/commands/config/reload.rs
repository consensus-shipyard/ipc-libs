// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::sdk::IpcAgentClient;
use async_trait::async_trait;
use clap::Args;

/// The command to reload the agent config after an update
pub(crate) struct ReloadConfig;

#[async_trait]
impl CommandLineHandler for ReloadConfig {
    type Arguments = ReloadConfigArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("reload config with args: {:?}", arguments);

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let client = IpcAgentClient::default_from_url(url);

        client.reload_config(arguments.path.clone()).await?;

        log::info!("Reload json rpc config successful");

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Reload config for IPC Agent JSON RPC server")]
pub(crate) struct ReloadConfigArgs {
    #[arg(
        short,
        long,
        help = "The path to ask json rpc server to load config from, optional"
    )]
    pub path: Option<String>,
    #[arg(short, long, help = "The JSON RPC server url for ipc agent, optional")]
    pub ipc_agent_url: Option<String>,
}