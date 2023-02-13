use std::collections::HashMap;
use std::fs;

use anyhow::{anyhow, Result};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
struct Config {
    subnets: HashMap<String, Subnet>,
}

#[derive(Deserialize)]
struct Subnet {
    path: String,
    rpc_api: Url,
    token: Option<String>,
    accounts: Option<Vec<String>>,
}

impl Config {
    fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[test]
fn read_toml() {
    let path = "config/example.toml";
    let config = Config::from_file(path).unwrap();
    println!("{:?}", config.subnets.keys());
    let c = &config.subnets["C"];
    println!("{}", c.rpc_api);
    let acc = c.accounts.as_ref().unwrap();
    let a = &acc[0];
    println!("{}", a);
}