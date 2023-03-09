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
use crate::server::handlers::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::{anyhow, Result};
pub use create::{CreateSubnetParams, CreateSubnetResponse};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use crate::server::handlers::join::JoinSubnetHandler;
use crate::server::handlers::kill::KillSubnetHandler;
use crate::server::handlers::leave::LeaveSubnetHandler;

pub type Method = String;

/// The collection of all json rpc handlers
pub struct Handlers {
    handlers: HashMap<Method, Box<dyn HandlerWrapper>>,
}

/// A util trait to avoid Box<dyn> and associated type mess in Handlers struct
#[async_trait]
trait HandlerWrapper: Send + Sync {
    async fn handle(&self, params: Value) -> Result<Value>;
}

#[async_trait]
impl <H: JsonRPCRequestHandler + Send + Sync> HandlerWrapper for H {
    async fn handle(&self, params: Value) -> Result<Value> {
        let p = serde_json::from_value(params)?;
        let r = self.handle(p).await?;
        Ok(serde_json::to_value(r)?)
    }
}

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
        let h: Box<dyn HandlerWrapper> = Box::new(ReloadConfigHandler::new(
            config.clone(),
            config_path_string,
        ));
        handlers.insert(String::from(json_rpc_methods::RELOAD_CONFIG),h);

        // subnet manager methods
        let pool = Arc::new(SubnetManagerPool::from_reload_config(config));
        let h: Box<dyn HandlerWrapper> = Box::new(CreateSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::CREATE_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(LeaveSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::LEAVE_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(KillSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::KILL_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(JoinSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::JOIN_SUBNET), h);

        Ok(Self { handlers })
    }

    pub async fn handle(&self, method: Method, params: Value) -> Result<Value> {
        if let Some(wrapper) = self.handlers.get(&method) {
            wrapper.handle(params).await
        } else {
            Err(anyhow!("method not supported"))
        }
    }
}
