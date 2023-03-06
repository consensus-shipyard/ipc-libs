//! The shared subnet manager module for all subnet management related RPC method calls.

use crate::config::Subnet;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::manager::LotusSubnetManager;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub(crate) struct SubnetManagerConnection<T: JsonRpcClient> {
    subnet: Subnet,
    manager: LotusSubnetManager<T>,
}

impl<T: JsonRpcClient> SubnetManagerConnection<T> {
    pub fn subnet(&self) -> &Subnet {
        &self.subnet
    }

    pub fn manager(&self) -> &LotusSubnetManager<T> {
        &self.manager
    }
}

/// The json rpc subnet manager connection pool. This struct can be shared by all the subnet methods.
/// As such, there is no need to re-init the same SubnetManager for different methods to reuse connections.
pub(crate) struct SubnetManagerPool<T: JsonRpcClient> {
    connections: HashMap<String, SubnetManagerConnection<T>>,
}

impl<T: JsonRpcClient + Send + Sync> SubnetManagerPool<T> {
    pub fn new(
        subnets: HashMap<String, Subnet>,
        mut managers: HashMap<String, LotusSubnetManager<T>>,
    ) -> Result<Self> {
        let mut connections = HashMap::new();

        // group the subnet config together with the corresponding subnet manager instance
        for (key, subnet) in subnets.into_iter() {
            let manager = managers
                .remove(&key)
                .ok_or_else(|| anyhow!("manager does not exist for all subnet"))?;
            connections.insert(key, SubnetManagerConnection { subnet, manager });
        }

        Ok(Self { connections })
    }

    pub fn contains_subnet(&self, subnet_str: &str) -> bool {
        self.connections.contains_key(subnet_str)
    }

    pub fn get(&self, subnet_str: &str) -> Option<&SubnetManagerConnection<T>> {
        self.connections.get(subnet_str)
    }
}

impl SubnetManagerPool<JsonRpcClientImpl> {
    pub fn new(
        subnets: HashMap<String, Subnet>,
    ) -> Result<Self> {
}