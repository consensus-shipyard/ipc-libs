//! Provides a simple way of reading configuration files.
//!
//! Reads a TOML config file for the IPC Agent and deserializes it in a type-safe way into a
//! [`Config`] struct. Usage:
//!
//! ```
//! let c: Config = Config::from_file("config/example.toml");
//! ```

use std::collections::HashMap;
use std::fmt::Formatter;
use std::fs;
use std::str::FromStr;

use anyhow::Result;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::de::{Error, SeqAccess};
use serde::{Deserialize, Deserializer};
use url::Url;

/// The top-level struct representing the config. Calls to [`Config::from_file`] deserialize into this
/// struct.
#[derive(Deserialize)]
pub(crate) struct Config {
    pub subnets: HashMap<String, Subnet>,
}

/// Represents a subnet declaration in the config.
#[derive(Deserialize)]
pub struct Subnet {
    #[serde(deserialize_with = "deserialize_path")]
    path: SubnetID,
    rpc_api: Url,
    auth_token: Option<String>,
    #[serde(deserialize_with = "deserialize_accounts", default)]
    accounts: Vec<Address>,
}

impl Config {
    /// Reads a TOML configuration file specified in the `path` and returns a [`Config`] struct.
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

/// A serde deserialization method to deserialize a subnet path string into a [`SubnetID`].
fn deserialize_path<'de, D>(deserializer: D) -> Result<SubnetID, D::Error>
where
    D: Deserializer<'de>,
{
    struct SubnetIDVisitor;
    impl<'de> serde::de::Visitor<'de> for SubnetIDVisitor {
        type Value = SubnetID;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: Error,
        {
            SubnetID::from_str(v).map_err(E::custom)
        }
    }
    deserializer.deserialize_str(SubnetIDVisitor)
}

/// A serde deserialization method to deserialize a list of account strings into a vector of
/// [`Address`].
fn deserialize_accounts<'de, D>(deserializer: D) -> Result<Vec<Address>, D::Error>
where
    D: Deserializer<'de>,
{
    struct AddressSeqVisitor;
    impl<'de> serde::de::Visitor<'de> for AddressSeqVisitor {
        type Value = Vec<Address>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec: Vec<Address> = Vec::new();
            while let Some(value) = seq.next_element::<String>()? {
                let a = Address::from_str(value.as_str()).map_err(Error::custom)?;
                vec.push(a);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_str(AddressSeqVisitor)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use fvm_shared::address::Address;
    use ipc_sdk::subnet_id::SubnetID;
    use url::Url;

    use crate::config::Config;

    #[test]
    fn read_example_config() {
        let path = "config/test_example.toml";
        let test_addr = "f3thgjtvoi65yzdcoifgqh6utjbaod3ukidxrx34heu34d6avx6z7r5766t5jqt42a44ehzcnw3u5ehz47n42a";
        let test_url = "https://example.org/rpc/v0";
        let root_path = "/root";
        let token_root = "token_root";
        let token_child = "token_child";
        let child_path = format!("{}/{}", "/root/", test_addr);

        let config = Config::from_file(path).unwrap();

        let root = &config.subnets["root"];
        assert_eq!(root.path, SubnetID::from_str(root_path).unwrap());
        assert_eq!(root.rpc_api, Url::from_str(test_url).unwrap());
        assert_eq!(root.auth_token.as_ref().unwrap(), token_root);
        assert_eq!(root.accounts.as_ref(), vec![]);

        let child = &config.subnets["child"];
        assert_eq!(child.path, SubnetID::from_str(child_path.as_str()).unwrap());
        assert_eq!(child.rpc_api, Url::from_str(test_url).unwrap());
        let address = Address::from_str(test_addr).unwrap();
        assert_eq!(child.auth_token.as_ref().unwrap(), token_child);
        assert_eq!(child.accounts.as_ref(), vec![address]);
    }
}
