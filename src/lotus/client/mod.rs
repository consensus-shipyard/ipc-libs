// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
mod ipc;
mod lotus;

use anyhow::Result;
use cid::Cid;
use fvm_shared::econ::TokenAmount;
use num_traits::cast::ToPrimitive;
use serde_json::json;

use crate::config::Subnet;
use crate::jsonrpc::{JsonRpcClient, JsonRpcClientImpl};
use crate::lotus::message::mpool::{
    MpoolPushMessage, MpoolPushMessageResponse, MpoolPushMessageResponseInner,
};
use crate::lotus::message::CIDMap;
use crate::lotus::StateWaitMsgResponse;

// RPC methods
mod methods {
    pub const MPOOL_PUSH_MESSAGE: &str = "Filecoin.MpoolPushMessage";
    pub const STATE_WAIT_MSG: &str = "Filecoin.StateWaitMsg";
    pub const STATE_NETWORK_NAME: &str = "Filecoin.StateNetworkName";
    pub const STATE_NETWORK_VERSION: &str = "Filecoin.StateNetworkVersion";
    pub const STATE_ACTOR_CODE_CIDS: &str = "Filecoin.StateActorCodeCIDs";
    pub const WALLET_NEW: &str = "Filecoin.WalletNew";
    pub const WALLET_LIST: &str = "Filecoin.WalletList";
    pub const WALLET_DEFAULT_ADDRESS: &str = "Filecoin.WalletDefaultAddress";
    pub const STATE_READ_STATE: &str = "Filecoin.StateReadState";
    pub const CHAIN_HEAD: &str = "Filecoin.ChainHead";
    pub const IPC_GET_PREV_CHECKPOINT_FOR_CHILD: &str = "Filecoin.IPCGetPrevCheckpointForChild";
    pub const IPC_GET_CHECKPOINT_TEMPLATE: &str = "Filecoin.IPCGetCheckpointTemplate";
    pub const IPC_READ_GATEWAY_STATE: &str = "Filecoin.IPCReadGatewayState";
    pub const IPC_READ_SUBNET_ACTOR_STATE: &str = "Filecoin.IPCReadSubnetActorState";
}

/// The default gateway actor address
const GATEWAY_ACTOR_ADDRESS: &str = "f064";

/// The struct implementation for Lotus Client API. It allows for multiple different trait
/// extension.
/// # Examples
/// ```no_run
/// use ipc_agent::{jsonrpc::JsonRpcClientImpl, lotus::LotusClient, lotus::client::LotusJsonRPCClient};
///
/// #[tokio::main]
/// async fn main() {
///     let h = JsonRpcClientImpl::new("<DEFINE YOUR URL HERE>".parse().unwrap(), None);
///     let n = LotusJsonRPCClient::new(h);
///     println!(
///         "wallets: {:?}",
///         n.wallet_new(ipc_agent::lotus::message::wallet::WalletKeyType::Secp256k1).await
///     );
/// }
/// ```
pub struct LotusJsonRPCClient<T: JsonRpcClient> {
    client: T,
}

impl<T: JsonRpcClient> LotusJsonRPCClient<T> {
    pub fn new(client: T) -> Self {
        Self { client }
    }
}

impl<T: JsonRpcClient + Send + Sync> LotusJsonRPCClient<T> {
    async fn mpool_push_message_inner(
        &self,
        msg: MpoolPushMessage,
    ) -> Result<MpoolPushMessageResponseInner> {
        let nonce = msg
            .nonce
            .map(|n| serde_json::Value::Number(n.into()))
            .unwrap_or(serde_json::Value::Null);

        let to_value = |t: Option<TokenAmount>| {
            t.map(|n| serde_json::Value::Number(n.atto().to_u64().unwrap().into()))
                .unwrap_or(serde_json::Value::Null)
        };
        let gas_limit = to_value(msg.gas_limit);
        let gas_premium = to_value(msg.gas_premium);
        let gas_fee_cap = to_value(msg.gas_fee_cap);
        let max_fee = to_value(msg.max_fee);

        // refer to: https://lotus.filecoin.io/reference/lotus/mpool/#mpoolpushmessage
        let params = json!([
            {
                "to": msg.to.to_string(),
                "from": msg.from.to_string(),
                "value": msg.value.atto().to_string(),
                "method": msg.method,
                "params": msg.params,

                // THESE ALL WILL AUTO POPULATE if null
                "nonce": nonce,
                "gas_limit": gas_limit,
                "gas_fee_cap": gas_fee_cap,
                "gas_premium": gas_premium,
                "cid": CIDMap::from(msg.cid),
                "version": serde_json::Value::Null,
            },
            {
                "max_fee": max_fee
            }
        ]);

        let r = self
            .client
            .request::<MpoolPushMessageResponse>(methods::MPOOL_PUSH_MESSAGE, params)
            .await?;
        log::debug!("received mpool_push_message response: {r:?}");

        Ok(r.message)
    }

    async fn state_wait_msg_inner(&self, cid: Cid, nonce: u64) -> Result<StateWaitMsgResponse> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statewaitmsg
        let params = json!([CIDMap::from(cid), nonce]);

        let r = self
            .client
            .request::<StateWaitMsgResponse>(methods::STATE_WAIT_MSG, params)
            .await?;
        log::debug!("received state_wait_msg response: {r:?}");
        Ok(r)
    }
}

impl LotusJsonRPCClient<JsonRpcClientImpl> {
    /// A constructor that returns a `LotusJsonRPCClient` from a `Subnet`. The returned
    /// `LotusJsonRPCClient` makes requests to the URL defined in the `Subnet`.
    pub fn from_subnet(subnet: &Subnet) -> Self {
        let url = subnet.jsonrpc_api_http.clone();
        let auth_token = subnet.auth_token.as_deref();
        let jsonrpc_client = JsonRpcClientImpl::new(url, auth_token);
        LotusJsonRPCClient::new(jsonrpc_client)
    }
}
