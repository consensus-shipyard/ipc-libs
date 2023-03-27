//! Set the validator memberships of a subnet command line operation

use async_trait::async_trait;
use clap::Args;
use std::fmt::Debug;

use crate::cli::commands::get_ipc_agent_url;
use crate::cli::{CommandLineHandler, GlobalArguments};
use crate::config::json_rpc_methods;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::server::SetMembershipParams;

/// The command to set the validator membership in a subnet
pub(crate) struct SetMemberships;

#[async_trait]
impl CommandLineHandler for SetMemberships {
    type Arguments = SetMembershipsArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!(
            "set validator memberships operation with args: {:?}",
            arguments
        );

        let url = get_ipc_agent_url(&arguments.ipc_agent_url, global)?;
        let json_rpc_client = JsonRpcClientImpl::new(url, None);

        let params = SetMembershipParams {
            subnet: arguments.subnet.clone(),
            gateway_addr: arguments.gateway_addr.clone(),
            validator_set: serde_json::from_str(&arguments.validator_set_json)?,
            from: arguments.from.clone(),
        };
        json_rpc_client
            .request::<()>(
                json_rpc_methods::SET_VALIDATOR_MEMBERSHIPS,
                serde_json::to_value(params)?,
            )
            .await?;

        log::info!(
            "updated subnet validator memberships: {:}",
            arguments.subnet
        );

        Ok(())
    }
}

#[derive(Debug, Args)]
#[command(about = "Set validator memberships in the gateway actor of a subnet")]
pub(crate) struct SetMembershipsArgs {
    #[arg(long, short, help = "The JSON RPC server url for ipc agent")]
    pub ipc_agent_url: Option<String>,
    #[arg(long, short, help = "The address that owns the message in the subnet")]
    pub from: Option<String>,
    #[arg(long, short, help = "The subnet to update validators")]
    pub subnet: String,
    #[arg(long, short, help = "The gateway actor address to update validators")]
    pub gateway_addr: Option<String>,
    /// The json string of validator set, using json might be easier as the payload is complex
    #[arg(long, short, help = "The json string of the validator set")]
    pub validator_set_json: String,
}
