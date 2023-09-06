// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::io::Write;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex};

use fvm_shared::address::Address;
use indoc::formatdoc;
use ipc_sdk::subnet_id::SubnetID;
use primitives::EthAddress;
use tempfile::NamedTempFile;
use url::Url;

use crate::config::{Config, ReloadableConfig};

// Arguments for the config's fields
const SERVER_JSON_RPC_ADDR: &str = "127.0.0.1:3030";
const ROOT_ID: &str = "/r123";
const CHILD_ID: &str = "/r123/f0100";
const GATEWAY_ADDR: &str = "f064";
const ROOT_AUTH_TOKEN: &str = "ROOT_AUTH_TOKEN";
const CHILD_AUTH_TOKEN: &str = "CHILD_AUTH_TOKEN";
const JSONRPC_API_HTTP: &str = "https://example.org/rpc/v0";
const PROVIDER_HTTP: &str = "http://127.0.0.1:3030/rpc/v1";
const ETH_ADDRESS: &str = "0x6be1ccf648c74800380d0520d797a170c808b624";
const ACCOUNT_ADDRESS: &str =
    "f3thgjtvoi65yzdcoifgqh6utjbaod3ukidxrx34heu34d6avx6z7r5766t5jqt42a44ehzcnw3u5ehz47n42a";

#[tokio::test]
async fn reload_works() {
    let config_str = config_str();

    let mut file = NamedTempFile::new().unwrap();
    let path = file
        .path()
        .as_os_str()
        .to_os_string()
        .into_string()
        .unwrap();

    file.write_all(config_str.as_bytes()).unwrap();

    let h = Arc::new(ReloadableConfig::new(path.clone()).unwrap());
    let original_config = h.get_config();

    // A simple barrier implementation for testing.
    // Refer to: https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/std/sync/struct.Condvar.html#examples
    // Only when the main thread has created a new subscriber then we trigger then update the config file.
    // This way, we dont miss the update and stuck the main thread.
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();
    let h_cloned = h.clone();
    tokio::spawn(async move {
        {
            let (lock, cvar) = &*pair;
            let mut started = lock.lock().unwrap();
            while !*started {
                started = cvar.wait(started).unwrap();
            }
        };

        let config_str = config_str_diff_addr();

        let mut file = file.reopen().unwrap();
        file.write_all(config_str.as_bytes()).unwrap();

        h_cloned.set_path(path);
        h_cloned.reload().await.unwrap();
    });

    let mut rx = h.new_subscriber();
    {
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
    }
    rx.recv().await.unwrap();

    let updated_config = h.get_config();
    assert_ne!(
        updated_config.server.json_rpc_address,
        original_config.server.json_rpc_address
    );
}

#[test]
fn check_server_config() {
    let config = read_config().server;
    assert_eq!(
        config.json_rpc_address,
        SocketAddr::from_str(SERVER_JSON_RPC_ADDR).unwrap(),
        "invalid server rpc address"
    );
}

#[test]
fn check_subnets_config() {
    let config = read_config().subnets;

    let rt_sn = SubnetID::from_str(ROOT_ID).unwrap();
    let root = &config[&rt_sn];
    assert_eq!(root.id, rt_sn);
    assert_eq!(root.network_name, "root");
    assert_eq!(
        root.gateway_addr(),
        Address::from_str(GATEWAY_ADDR).unwrap()
    );
    assert_eq!(*root.rpc_http(), Url::from_str(JSONRPC_API_HTTP).unwrap());
    assert_eq!(root.auth_token().as_ref().unwrap(), ROOT_AUTH_TOKEN);

    let child_id = SubnetID::from_str(CHILD_ID).unwrap();
    let child = &config[&child_id];
    assert_eq!(child.id, child_id);
    assert_eq!(child.network_name, "child");
    assert_eq!(
        child.gateway_addr(),
        Address::from(EthAddress::from_str(ETH_ADDRESS).unwrap())
    );
    assert_eq!(*child.rpc_http(), Url::from_str(PROVIDER_HTTP).unwrap(),);
    assert_eq!(child.auth_token().as_ref().unwrap(), CHILD_AUTH_TOKEN);
    assert_eq!(
        child.accounts(),
        vec![
            Address::from(EthAddress::from_str(ETH_ADDRESS).unwrap()),
            Address::from(EthAddress::from_str(ETH_ADDRESS).unwrap())
        ],
    );
}

fn config_str() -> String {
    formatdoc!(
        r#"
        [server]
        json_rpc_address = "{SERVER_JSON_RPC_ADDR}"

        [[subnets]]
        id = "{ROOT_ID}"
        network_name = "root"

        [subnets.config]
        network_type = "fvm"
        gateway_addr = "{GATEWAY_ADDR}"
        jsonrpc_api_http = "{JSONRPC_API_HTTP}"
        auth_token = "{ROOT_AUTH_TOKEN}"
        accounts = ["{ACCOUNT_ADDRESS}"]

        [[subnets]]
        id = "{CHILD_ID}"
        network_name = "child"

        [subnets.config]
        network_type = "fevm"
        auth_token = "{CHILD_AUTH_TOKEN}"
        provider_http = "{PROVIDER_HTTP}"
        registry_addr = "{ETH_ADDRESS}"
        gateway_addr = "{ETH_ADDRESS}"
        accounts = ["{ETH_ADDRESS}", "{ETH_ADDRESS}"]
        "#
    )
}

fn config_str_diff_addr() -> String {
    formatdoc!(
        r#"
        [server]
        json_rpc_address = "127.0.0.1:3031"

        [[subnets]]
        id = "{ROOT_ID}"
        network_name = "root"

        [subnets.config]
        network_type = "fvm"
        gateway_addr = "{GATEWAY_ADDR}"
        jsonrpc_api_http = "{JSONRPC_API_HTTP}"
        auth_token = "{ROOT_AUTH_TOKEN}"
        accounts = ["{ACCOUNT_ADDRESS}"]

        [[subnets]]
        id = "{CHILD_ID}"
        network_name = "child"

        [subnets.config]
        network_type = "fevm"
        auth_token = "{CHILD_AUTH_TOKEN}"
        provider_http = "{PROVIDER_HTTP}"
        registry_addr = "{ETH_ADDRESS}"
        gateway_addr = "{ETH_ADDRESS}"
        accounts = ["{ETH_ADDRESS}", "{ETH_ADDRESS}"]
        "#
    )
}

fn read_config() -> Config {
    Config::from_toml_str(config_str().as_str()).unwrap()
}