//! The module contains the handlers implementation for the json rpc server.

pub mod create;
mod subnet;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
pub use create::{CreateSubnetResponse, CreateSubnetParams};
use crate::config::Subnet;
use crate::server::create::CreateSubnetHandler;
use crate::server::handlers::subnet::SubnetManagerShared;
use crate::server::JsonRPCRequestHandler;

pub type Method = String;

/// A util enum to avoid Box<dyn> mess in Handlers struct
enum HandlerWrapper {
    CreateSubnet(CreateSubnetHandler),
}

/// The collection of all json rpc handlers
pub struct Handlers {
    handlers: HashMap<Method, HandlerWrapper>,
}

impl Handlers {
    pub fn construct(subnets: HashMap<String, Subnet>) -> Self {
        let mut handlers = HashMap::new();

        let shared = Arc::new(SubnetManagerShared::new(subnets));

        let create_subnet = HandlerWrapper::CreateSubnet(CreateSubnetHandler::new(shared));
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
}