//! Provides a simple way of reading configuration files.
//!
//! Reads a TOML config file for the IPC Agent and deserializes it in a type-safe way into a
//! [`Config`] struct.

mod deserialize;

use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;

use anyhow::Result;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::Deserialize;
use url::Url;
use crate::config::deserialize::{deserialize_subnet_id, deserialize_accounts};

/// The top-level struct representing the config. Calls to [`Config::from_file`] deserialize into
/// this struct.
#[derive(Deserialize)]
pub(crate) struct Config {
    pub server: Server,
    pub subnets: HashMap<String, Subnet>,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub json_rpc_address: SocketAddr,
}

/// Represents a subnet declaration in the config.
#[derive(Deserialize)]
pub struct Subnet {
    #[serde(deserialize_with = "deserialize_subnet_id")]
    id: SubnetID,
    jsonrpc_api_http: Url,
    jsonrpc_api_ws: Option<Url>,
    auth_token: Option<String>,
    #[serde(deserialize_with = "deserialize_accounts", default)]
    accounts: Vec<Address>,
}

impl Config {
    /// Reads a TOML configuration in the `s` string and returns a [`Config`] struct.
    pub fn from_str(s: &str) -> Result<Self> {
        let config = toml::from_str(&s)?;
        Ok(config)
    }

    /// Reads a TOML configuration file specified in the `path` and returns a [`Config`] struct.
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = Config::from_str(contents.as_str())?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::str::FromStr;

    use fvm_shared::address::Address;
    use indoc::formatdoc;
    use ipc_sdk::subnet_id::{ROOTNET_ID, SubnetID};
    use url::Url;

    use crate::config::Config;

    #[test]
    fn read_config() {
        // Arguments for the config's fields
        let server_json_rpc_addr = "127.0.0.1:3030";

        let root_id = "/root";
        let child_id = "/root/f0100";
        let root_auth_token = "root_auth_token";
        let child_auth_token = "child_auth_token";
        let jsonrpc_api_http = "https://example.org/rpc/v0";
        let jsonrpc_api_ws = "ws://example.org/rpc/v0";
        let account_address = "f3thgjtvoi65yzdcoifgqh6utjbaod3ukidxrx34heu34d6avx6z7r5766t5jqt42a44ehzcnw3u5ehz47n42a";

        let config_str = formatdoc!(
            r#"
            [server]
            json_rpc_address = "{server_json_rpc_addr}"

            [subnets]

            [subnets.root]
            id = "{root_id}"
            jsonrpc_api_http = "{jsonrpc_api_http}"
            jsonrpc_api_ws = "{jsonrpc_api_ws}"
            auth_token = "{root_auth_token}"

            [subnets.child]
            id = "{child_id}"
            jsonrpc_api_http = "{jsonrpc_api_http}"
            auth_token = "{child_auth_token}"
            accounts = ["{account_address}"]
        "#
        );

        println!("{}", config_str);
        let config = Config::from_str(config_str.as_str()).unwrap();

        assert_eq!(
            config.server.json_rpc_address,
            SocketAddr::from_str(server_json_rpc_addr).unwrap()
        );

        let root = &config.subnets["root"];
        assert_eq!(root.id, *ROOTNET_ID);
        assert_eq!(
            root.jsonrpc_api_http,
            Url::from_str(jsonrpc_api_http).unwrap()
        );
        assert_eq!(
            root.jsonrpc_api_ws.as_ref().unwrap(),
            &Url::from_str(jsonrpc_api_ws).unwrap()
        );
        assert_eq!(root.auth_token.as_ref().unwrap(), root_auth_token);

        let child = &config.subnets["child"];
        assert_eq!(child.id, SubnetID::from_str(child_id).unwrap());
        assert_eq!(
            child.jsonrpc_api_http,
            Url::from_str(jsonrpc_api_http).unwrap()
        );
        assert_eq!(child.auth_token.as_ref().unwrap(), child_auth_token);
        assert_eq!(
            child.accounts.as_ref(),
            vec![Address::from_str(account_address).unwrap()]
        );
    }
}
