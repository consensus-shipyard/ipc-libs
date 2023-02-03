use crate::jsonrpc::types::{MpoolPushMessage, MpoolPushMessageResponse};
use crate::jsonrpc::{CIDMap, JsonRpcClient};
use anyhow::{anyhow, Result};
use cid::Cid;
use fvm_shared::econ::TokenAmount;
use num_traits::cast::ToPrimitive;
use serde::de::DeserializeOwned;
use serde_json::json;

const DEFAULT_VERSION: u16 = 42;
const MESSAGE_KEY: &str = "Message";

// RPC endpoints
mod endpoints {
    pub const MEM_PUSH_MESSAGE_ENDPOINT: &str = "Filecoin.MpoolPushMessage";
}

pub struct LotusApi<Inner: JsonRpcClient> {
    inner: Inner,
}

impl<Inner: JsonRpcClient> LotusApi<Inner> {
    pub fn new(inner: Inner) -> Self {
        Self { inner }
    }

    pub async fn mpool_push_message(&self, msg: MpoolPushMessage) -> Result<Cid> {
        let from = msg.from;

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
                "version": DEFAULT_VERSION
            },
            {
                "max_fee": max_fee
            }
        ]);

        let r = self
            .inner
            .request(endpoints::MEM_PUSH_MESSAGE_ENDPOINT, to_send)
            .await?;
        log::debug!("received response: {r:}");
        let m = parse_response::<MpoolPushMessageResponse>(r.get(MESSAGE_KEY).unwrap().clone())?;
        m.get_root_cid().ok_or_else(|| anyhow!("No cid in result"))
    }
}

fn parse_response<T: DeserializeOwned>(r: serde_json::Value) -> Result<T> {
    let message = r
        .get("Message")
        .ok_or_else(|| anyhow!("Invalid response"))?;
    let message =
        serde_json::from_value(message.clone()).map_err(|_| anyhow!("Cannot parse response"))?;
    Ok(message)
}
