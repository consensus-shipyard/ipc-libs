// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::io::Write;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use fvm_shared::address::Address;
use indoc::formatdoc;
use ipc_sdk::subnet_id::{SubnetID, ROOTNET_ID};
use tempfile::NamedTempFile;
use url::Url;

use crate::config::{Config, HotReloadingConfig};

// Arguments for the config's fields
const SERVER_JSON_RPC_ADDR: &str = "127.0.0.1:3030";
const ROOT_ID: &str = "/root";
const CHILD_ID: &str = "/root/f0100";
const ROOT_AUTH_TOKEN: &str = "ROOT_AUTH_TOKEN";
const CHILD_AUTH_TOKEN: &str = "CHILD_AUTH_TOKEN";
const JSONRPC_API_HTTP: &str = "https://example.org/rpc/v0";
const JSONRPC_API_WS: &str = "ws://example.org/rpc/v0";
const ACCOUNT_ADDRESS: &str =
    "f3thgjtvoi65yzdcoifgqh6utjbaod3ukidxrx34heu34d6avx6z7r5766t5jqt42a44ehzcnw3u5ehz47n42a";

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
fn reload_works() {
    let config_str = config_str();
    let original_config = Config::from_toml_str(&config_str).unwrap();

    let mut file = NamedTempFile::new().unwrap();
    let path = file
        .path()
        .as_os_str()
        .to_os_string()
        .into_string()
        .unwrap();

    file.write_all(config_str.as_bytes()).unwrap();

    let interval = 1;

    let h = HotReloadingConfig::new_with_watcher(path, interval).unwrap();
    h.read_from_config(|config| {
        assert_eq!(
            config.server.json_rpc_address,
            original_config.server.json_rpc_address
        );
    });

    let (tx, rx) = channel();
    let t1 = thread::spawn(move || {
        let mut loop_num = 1;

        // we need to loop here because we need to make sure the
        // config update is picked up by the watcher thread.
        // It is possible that when the `file.write_all` is done,
        // `wait_for_modification` has not yet being called. Need
        // to loop and update to make sure the update is propagated.
        loop {
            let config_str = config_str_diff_addr(loop_num);
            loop_num += 1;

            let mut file = file.reopen().unwrap();
            file.write_all(config_str.as_bytes()).unwrap();

            match rx.try_recv() {
                Ok(_) => break,
                Err(_) => {
                    sleep(Duration::from_secs(1));
                }
            }
        }
    });

    loop {
        let is_modified = h.try_wait_modification().unwrap();
        if is_modified {
            println!("config modification detected");
            break;
        }
    }

    let mut addr = None;
    h.read_from_config(|config| {
        addr = Some(config.server.json_rpc_address);
    });
    h.stop_watcher();
    tx.send(()).unwrap_or_default();

    let addr = addr.unwrap();
    assert_ne!(addr, original_config.server.json_rpc_address);

    t1.join().unwrap();
}

#[test]
fn check_subnets_config() {
    let config = read_config().subnets;

    let root = &config["root"];
    assert_eq!(root.id, *ROOTNET_ID);
    assert_eq!(
        root.jsonrpc_api_http,
        Url::from_str(JSONRPC_API_HTTP).unwrap()
    );
    assert_eq!(
        root.jsonrpc_api_ws.as_ref().unwrap(),
        &Url::from_str(JSONRPC_API_WS).unwrap()
    );
    assert_eq!(root.auth_token.as_ref().unwrap(), ROOT_AUTH_TOKEN);

    let child = &config["child"];
    assert_eq!(child.id, SubnetID::from_str(CHILD_ID).unwrap(),);
    assert_eq!(
        child.jsonrpc_api_http,
        Url::from_str(JSONRPC_API_HTTP).unwrap(),
    );
    assert_eq!(child.auth_token.as_ref().unwrap(), CHILD_AUTH_TOKEN,);
    assert_eq!(
        child.accounts.as_ref(),
        vec![Address::from_str(ACCOUNT_ADDRESS).unwrap()],
    );
}

fn config_str() -> String {
    formatdoc!(
        r#"
            [server]
            json_rpc_address = "{SERVER_JSON_RPC_ADDR}"

            [subnets]

            [subnets.root]
            id = "{ROOT_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            jsonrpc_api_ws = "{JSONRPC_API_WS}"
            auth_token = "{ROOT_AUTH_TOKEN}"

            [subnets.child]
            id = "{CHILD_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            auth_token = "{CHILD_AUTH_TOKEN}"
            accounts = ["{ACCOUNT_ADDRESS}"]
        "#
    )
}

fn config_str_diff_addr(idx: u8) -> String {
    formatdoc!(
        r#"
            [server]
            json_rpc_address = "127.0.0.1:303{idx}"

            [subnets]

            [subnets.root]
            id = "{ROOT_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            jsonrpc_api_ws = "{JSONRPC_API_WS}"
            auth_token = "{ROOT_AUTH_TOKEN}"

            [subnets.child]
            id = "{CHILD_ID}"
            jsonrpc_api_http = "{JSONRPC_API_HTTP}"
            auth_token = "{CHILD_AUTH_TOKEN}"
            accounts = ["{ACCOUNT_ADDRESS}"]
        "#
    )
}

fn read_config() -> Config {
    let config_str = config_str();
    Config::from_toml_str(config_str.as_str()).unwrap()
}
