use std::net::SocketAddr;
use fvm_shared::address::Network;
use serde::Deserialize;
use crate::config::deserialize::deserialize_network;

pub const JSON_RPC_ENDPOINT: &str = "json_rpc";

#[derive(Deserialize, Clone)]
pub struct Server {
    pub json_rpc_address: SocketAddr,
    #[serde(deserialize_with = "deserialize_network")]
    pub network: Network
}