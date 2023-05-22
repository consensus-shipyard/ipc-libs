// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use ipc_sdk::subnet_id::SubnetID;
use std::str::FromStr;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

mod infra;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let eudico_binary_path =
        std::env::var("EUDICO_BIN").unwrap_or_else(|_| "/home/admin/lotus/eudico".to_string());
    let ipc_root_folder =
        std::env::var("IPC_ROOT_FOLDER").unwrap_or_else(|_| "/home/admin/.ipc-agent".to_string());
    let root_lotus_path = std::env::var("ROOT_LOTUS_PATH")
        .unwrap_or_else(|_| "/home/admin/.lotus-local-net0".to_string());

    let api_port_sequence = Arc::new(AtomicU16::new(10));
    let mut topology = infra::SubnetTopology::new(
        "test-subnet-1".to_string(),
        "t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq".to_string(),
        root_lotus_path,
        ipc_root_folder,
        2,
        eudico_binary_path,
        SubnetID::from_str("/root").unwrap(),
        api_port_sequence,
    );

    let r = infra::subnet::spawn_child_subnet(&mut topology).await;
    if r.is_err() {
        log::error!("cannot spawn subnet: {:}", r.unwrap_err());
    } else {
        log::info!("subnet created, sleep");
        sleep(Duration::from_secs(30));
    }
}
