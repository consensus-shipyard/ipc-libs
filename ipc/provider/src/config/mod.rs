// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Provides a simple way of reading configuration files.
//!
//! Reads a TOML config file for the IPC Agent and deserializes it in a type-safe way into a
//! [`Config`] struct.

pub mod deserialize;
mod reload;
pub mod subnet;

pub mod serialize;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use deserialize::deserialize_subnets_from_vec;
use ipc_sdk::subnet_id::SubnetID;
pub use reload::ReloadableConfig;
use serde::{Deserialize, Serialize};
use serialize::serialize_subnets_to_str;
pub use subnet::Subnet;

pub const JSON_RPC_VERSION: &str = "2.0";

/// DefaulDEFAULT_CHAIN_IDSUBNET_e
pub const DEFAULT_CONFIG_TEMPLATE: &str = r#"
keystore_path = "~/.ipc"

# Filecoin Calibration
[[subnets]]
id = "/r314159"

[subnets.config]
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
gateway_addr = "0x0341fA160C66aBB112195192aE359a6D61df45cd"
registry_addr = "0xc7068Cea947035560128a6a6F4c8913523A5A44C"

# Mycelium Calibration
[[subnets]]
id = "/r314159/t410fnotsxwgnxcjp5phjmgp6n3lnhxvrf3pncnm3oxq"

[subnets.config]
network_type = "fevm"
provider_http = "https://api.mycelium.calibration.node.glif.io/"
gateway_addr = "0x77aa40b105843728088c0132e43fc44348881da8"
registry_addr = "0x74539671a1d2f1c8f200826baba665179f53a1b7"

# Subnet template - uncomment and adjust before using
# [[subnets]]
# id = "/r314159/<SUBNET_ID>"

# [subnets.config]
# network_type = "fevm"
# provider_http = "https://<RPC_ADDR>/"
# gateway_addr = "0x77aa40b105843728088c0132e43fc44348881da8"
# registry_addr = "0x74539671a1d2f1c8f200826baba665179f53a1b7"
"#;

/// The top-level struct representing the config. Calls to [`Config::from_file`] deserialize into
/// this struct.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Config {
    /// Directory of the keystore that wants to be made available by the provider.
    pub keystore_path: Option<String>,
    #[serde(deserialize_with = "deserialize_subnets_from_vec", default)]
    #[serde(serialize_with = "serialize_subnets_to_str")]
    pub subnets: HashMap<SubnetID, Subnet>,
}

impl Config {
    /// Returns an empty config to be populated further
    pub fn new() -> Self {
        Config {
            keystore_path: None,
            subnets: Default::default(),
        }
    }

    /// Reads a TOML configuration in the `s` string and returns a [`Config`] struct.
    pub fn from_toml_str(s: &str) -> Result<Self> {
        let config = toml::from_str(s)?;
        Ok(config)
    }

    /// Reads a TOML configuration file specified in the `path` and returns a [`Config`] struct.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = Config::from_toml_str(contents.as_str())?;
        Ok(config)
    }

    /// Reads a TOML configuration file specified in the `path` and returns a [`Config`] struct.
    pub async fn from_file_async(path: impl AsRef<Path>) -> Result<Self> {
        let contents = tokio::fs::read_to_string(path).await?;
        Config::from_toml_str(contents.as_str())
    }

    pub async fn write_to_file_async(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = toml::to_string(self)?;
        tokio::fs::write(path, content.into_bytes()).await?;
        Ok(())
    }

    pub fn add_subnet(&mut self, subnet: Subnet) {
        self.subnets.insert(subnet.id.clone(), subnet);
    }

    pub fn remove_subnet(&mut self, subnet_id: &SubnetID) {
        self.subnets.remove(subnet_id);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
