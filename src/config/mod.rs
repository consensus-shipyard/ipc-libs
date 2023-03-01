//! Provides a simple way of reading configuration files.
//!
//! Reads a TOML config file for the IPC Agent and deserializes it in a type-safe way into a
//! [`Config`] struct.

mod deserialize;
mod server;
mod subnet;
mod hot;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
pub use server::Server;
pub use subnet::Subnet;
pub use server::JSON_RPC_ENDPOINT;
pub use hot::HotReloadingConfig;

pub const JSON_RPC_VERSION: &str = "2.0";
pub const IPC_GATEWAY_ADDR: u64 = 64;

/// The top-level struct representing the config. Calls to [`Config::from_file`] deserialize into
/// this struct.
#[derive(Deserialize, Clone)]
pub struct Config {
    pub server: Server,
    pub subnets: HashMap<String, Subnet>,
}

impl Config {
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
}
