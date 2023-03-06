// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The module contains the handlers implementation for the json rpc server.

pub mod create;
mod subnet;

use crate::config::Subnet;
use crate::jsonrpc::JsonRpcClientImpl;
use crate::lotus::client::LotusJsonRPCClient;
use crate::manager::LotusSubnetManager;
use crate::server::create::CreateSubnetHandler;
use crate::server::handlers::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::{anyhow, Result};
pub use create::{CreateSubnetParams, CreateSubnetResponse};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub type Method = String;

/// A util enum to avoid Box<dyn> mess in Handlers struct
enum HandlerWrapper {
    CreateSubnet(CreateSubnetHandler<JsonRpcClientImpl>),
}

/// The collection of all json rpc handlers
pub struct Handlers {
    handlers: HashMap<Method, HandlerWrapper>,
}

impl Handlers {
    pub fn new(subnets: HashMap<String, Subnet>) -> Self {
        let managers = Self::create_managers(&subnets);
        let pool = Arc::new(
            SubnetManagerPool::new(subnets, managers)
                .expect("cannot init subnet managers, configuration error"),
        );

        let mut handlers = HashMap::new();

        let create_subnet = HandlerWrapper::CreateSubnet(CreateSubnetHandler::new(pool));
        handlers.insert(String::from("create_subnet"), create_subnet);

        Self { handlers }
    }

    pub async fn handle(&self, method: Method, params: Value) -> Result<Value> {
        if let Some(wrapper) = self.handlers.get(&method) {
            match wrapper {
                HandlerWrapper::CreateSubnet(handler) => {
                    let r = handler.handle(serde_json::from_value(params)?).await?;
                    Ok(serde_json::to_value(r)?)
                }
            }
        } else {
            Err(anyhow!("method not supported"))
        }
    }

    /// Create the needed subnet managers for each subnet.
    ///
    /// Since we don't have a large number of subnet for now, to keep things simple,
    /// these managers are created upon initialization.
    ///
    /// If the traffic received by the json rpc node increases or the number of subnets increases
    /// significantly, we can use Connection Pooling for manage the subnet managers.
    fn create_managers(
        subnets: &HashMap<String, Subnet>,
    ) -> HashMap<String, LotusSubnetManager<JsonRpcClientImpl>> {
        let mut managers = HashMap::new();
        subnets.iter().for_each(|(subnet, subnet_config)| {
            let json_rpc_client = JsonRpcClientImpl::new(
                subnet_config.jsonrpc_api_http.clone(),
                subnet_config.auth_token.clone().as_deref(),
            );
            let lotus_client = LotusJsonRPCClient::new(json_rpc_client);
            managers.insert(subnet.clone(), LotusSubnetManager::new(lotus_client));
        });
        managers
    }
}
