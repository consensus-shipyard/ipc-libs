// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The module contains the handlers implementation for the json rpc server.

mod config;
pub mod create;
mod join;
mod kill;
mod leave;
mod subnet;

use crate::config::json_rpc_methods;
use crate::config::ReloadableConfig;
use crate::server::create::CreateSubnetHandler;
use crate::server::handlers::config::ReloadConfigHandler;
use crate::server::handlers::join::JoinSubnetHandler;
use crate::server::handlers::kill::KillSubnetHandler;
use crate::server::handlers::leave::LeaveSubnetHandler;
use crate::server::handlers::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::{anyhow, Result};
pub use create::{CreateSubnetParams, CreateSubnetResponse};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// A util macro to create the handlers and do the method routing.
/// This macro generates the `handle` method. In the method, it just matches the handler with
/// different enum types to get the inner handler instance. Every time a new handler is added,
/// a new match statement must be copy-pasted. With this macro, we can reduce a lot of repetitive
/// copy paste.
macro_rules! create_handlers {
    (enum HandlerWrapper { $($name:tt($handler:tt),)* }) => {
        /// A util enum to avoid Box<dyn> mess in Handlers struct
        enum HandlerWrapper {
            $(
            $name($handler)
            ),*
        }

        impl Handlers {
            pub async fn handle(&self, method: Method, params: Value) -> Result<Value> {
                if let Some(wrapper) = self.handlers.get(&method) {
                    match wrapper {
                        $(
                            HandlerWrapper::$name(handler) => {
                                let r = handler.handle(serde_json::from_value(params)?).await?;
                                Ok(serde_json::to_value(r)?)
                            }
                        ),*
                    }
                } else {
                    Err(anyhow!("method not supported"))
                }
            }
        }
    };
    (enum HandlerWrapper { $($name:tt($handler:tt)),* }) => {
        create_handlers!(enum HandlerWrapper { $($name($handler),)* });
    }
}

pub type Method = String;

/// The collection of all json rpc handlers
pub struct Handlers {
    handlers: HashMap<Method, HandlerWrapper>,
}

// Create the handler wrapper
create_handlers!(
    enum HandlerWrapper {
        CreateSubnet(CreateSubnetHandler),
        JoinSubnet(JoinSubnetHandler),
        LeaveSubnet(LeaveSubnetHandler),
        KillSubnet(KillSubnetHandler),
        ReloadConfig(ReloadConfigHandler),
    }
);

impl Handlers {
    /// We test the handlers separately and individually instead of from the handlers.
    /// Convenient method for json rpc to test routing.
    #[cfg(test)]
    pub fn empty_handlers() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn new(config_path_string: String) -> Result<Self> {
        let mut handlers = HashMap::new();

        let config = Arc::new(ReloadableConfig::new(config_path_string.clone())?);
        handlers.insert(
            String::from(json_rpc_methods::RELOAD_CONFIG),
            HandlerWrapper::ReloadConfig(ReloadConfigHandler::new(
                config.clone(),
                config_path_string,
            )),
        );

        // subnet manager methods
        let pool = Arc::new(SubnetManagerPool::from_reload_config(config));
        handlers.insert(
            String::from(json_rpc_methods::CREATE_SUBNET),
            HandlerWrapper::CreateSubnet(CreateSubnetHandler::new(pool.clone())),
        );
        handlers.insert(
            String::from(json_rpc_methods::CREATE_SUBNET),
            HandlerWrapper::LeaveSubnet(LeaveSubnetHandler::new(pool.clone())),
        );
        handlers.insert(
            String::from(json_rpc_methods::CREATE_SUBNET),
            HandlerWrapper::KillSubnet(KillSubnetHandler::new(pool.clone())),
        );
        handlers.insert(
            String::from(json_rpc_methods::JOIN_SUBNET),
            HandlerWrapper::JoinSubnet(JoinSubnetHandler::new(pool)),
        );

        Ok(Self { handlers })
    }
}
