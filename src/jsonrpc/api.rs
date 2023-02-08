use crate::jsonrpc::types::{MpoolPushMessage, MpoolPushMessageInner};
use crate::jsonrpc::{CIDMap, JsonRpcClient, StateWaitMsgResponse};
use crate::{MpoolPushMessageResponse, ReadStateResponse, WalletKeyType, WalletListResponse};
use anyhow::Result;
use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use num_traits::cast::ToPrimitive;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::fmt::Debug;
use std::str::FromStr;

// RPC endpoints
mod endpoints {
    pub const MEM_PUSH_MESSAGE_ENDPOINT: &str = "Filecoin.MpoolPushMessage";
    pub const STATE_WAIT_MSG: &str = "Filecoin.StateWaitMsg";
    pub const WALLET_NEW: &str = "Filecoin.WalletNew";
    pub const WALLET_LIST: &str = "Filecoin.WalletList";
    pub const WALLET_DEFAULT_ADDRESS: &str = "Filecoin.WalletDefaultAddress";
    pub const READ_STATE: &str = "Filecoin.StateReadState";
}

pub struct LotusApi<Inner: JsonRpcClient> {
    inner: Inner,
}

impl<Inner: JsonRpcClient> LotusApi<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self { inner }
    }

    pub async fn mpool_push_message(&self, msg: MpoolPushMessage) -> Result<MpoolPushMessageInner> {
        let from = if let Some(f) = msg.from {
            f
        } else {
            self.wallet_default().await?
        };

        let nonce = msg
            .nonce
            .map(|n| serde_json::Value::Number(n.into()))
            .unwrap_or(serde_json::Value::Null);

        let f = |t: Option<TokenAmount>| {
            t.map(|n| serde_json::Value::Number(n.atto().to_u64().unwrap().into()))
                .unwrap_or(serde_json::Value::Null)
        };
        let gas_limit = f(msg.gas_limit);
        let gas_premium = f(msg.gas_premium);
        let gas_fee_cap = f(msg.gas_fee_cap);
        let max_fee = f(msg.max_fee);

        // refer to: https://lotus.filecoin.io/reference/lotus/mpool/#mpoolpushmessage
        let to_send = json!([
            {
                "to": msg.to.to_string(),
                "from": from.to_string(),
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
            .inner
            .request::<MpoolPushMessageResponse>(endpoints::MEM_PUSH_MESSAGE_ENDPOINT, to_send)
            .await?;
        log::debug!("received response: {r:?}");

        Ok(r.message)
    }

    pub async fn state_wait_msg(&self, cid: Cid, nonce: u64) -> Result<StateWaitMsgResponse> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statewaitmsg
        let to_send = json!([CIDMap::from(cid), nonce]);

        let r = self
            .inner
            .request::<StateWaitMsgResponse>(endpoints::STATE_WAIT_MSG, to_send)
            .await?;
        log::debug!("received response: {r:?}");
        Ok(r)
    }

    pub async fn wallet_default(&self) -> Result<Address> {
        // refer to: https://lotus.filecoin.io/reference/lotus/wallet/#walletdefaultaddress
        let r = self
            .inner
            .request::<String>(endpoints::WALLET_DEFAULT_ADDRESS, json!({}))
            .await?;
        log::debug!("received response: {r:?}");

        let addr = Address::from_str(&r)?;
        Ok(addr)
    }

    pub async fn wallet_list(&self) -> Result<WalletListResponse> {
        // refer to: https://lotus.filecoin.io/reference/lotus/wallet/#walletlist
        let r = self
            .inner
            .request::<WalletListResponse>(endpoints::WALLET_LIST, json!({}))
            .await?;
        log::debug!("received response: {r:?}");
        Ok(r)
    }

    pub async fn wallet_new(&self, key_type: WalletKeyType) -> Result<String> {
        let s = key_type.as_ref();
        // refer to: https://lotus.filecoin.io/reference/lotus/wallet/#walletnew
        let r = self
            .inner
            .request::<String>(endpoints::WALLET_NEW, json!([s]))
            .await?;
        log::debug!("received response: {r:?}");
        Ok(r)
    }

    pub async fn read_state<State: DeserializeOwned + Debug>(
        &self,
        address: Address,
        tipset: Cid,
    ) -> Result<ReadStateResponse<State>> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statereadstate
        let r = self
            .inner
            .request::<ReadStateResponse<State>>(
                endpoints::READ_STATE,
                json!([address.to_string(), [CIDMap::from(tipset)]]),
            )
            .await?;
        log::debug!("received response: {r:?}");
        Ok(r)
    }
}
