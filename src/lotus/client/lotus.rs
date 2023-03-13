use crate::jsonrpc::{JsonRpcClient, NO_PARAMS};
use crate::lotus::client::{methods, LotusJsonRPCClient};
use crate::lotus::message::chain::ChainHeadResponse;
use crate::lotus::message::mpool::{MpoolPushMessage, MpoolPushMessageResponseInner};
use crate::lotus::message::state::{ReadStateResponse, StateWaitMsgResponse};
use crate::lotus::message::wallet::{WalletKeyType, WalletListResponse};
use crate::lotus::message::CIDMap;
use crate::lotus::{LotusClient, NetworkVersion};
use async_trait::async_trait;
use cid::Cid;
use fvm_shared::address::Address;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;

#[async_trait]
impl<T: JsonRpcClient + Send + Sync> LotusClient for LotusJsonRPCClient<T> {
    async fn mpool_push_message(
        &self,
        msg: MpoolPushMessage,
    ) -> anyhow::Result<MpoolPushMessageResponseInner> {
        self.mpool_push_message_inner(msg).await
    }

    async fn state_wait_msg(&self, cid: Cid, nonce: u64) -> anyhow::Result<StateWaitMsgResponse> {
        self.state_wait_msg_inner(cid, nonce).await
    }

    async fn state_network_name(&self) -> anyhow::Result<String> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statenetworkname
        let r = self
            .client
            .request::<String>(methods::STATE_NETWORK_NAME, serde_json::Value::Null)
            .await?;
        log::debug!("received state_network_name response: {r:?}");
        Ok(r)
    }

    async fn state_network_version(&self, tip_sets: Vec<Cid>) -> anyhow::Result<NetworkVersion> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statenetworkversion
        let params = json!([tip_sets.into_iter().map(CIDMap::from).collect::<Vec<_>>()]);

        let r = self
            .client
            .request::<NetworkVersion>(methods::STATE_NETWORK_VERSION, params)
            .await?;

        log::debug!("received state_network_version response: {r:?}");
        Ok(r)
    }

    async fn state_actor_code_cids(
        &self,
        network_version: NetworkVersion,
    ) -> anyhow::Result<HashMap<String, Cid>> {
        // refer to: https://github.com/filecoin-project/lotus/blob/master/documentation/en/api-v1-unstable-methods.md#stateactormanifestcid
        let params = json!([network_version]);

        let r = self
            .client
            .request::<HashMap<String, CIDMap>>(methods::STATE_ACTOR_CODE_CIDS, params)
            .await?;

        let mut cids = HashMap::new();
        for (key, cid_map) in r.into_iter() {
            cids.insert(key, Cid::try_from(cid_map)?);
        }

        log::debug!("received state_actor_manifest_cid response: {cids:?}");
        Ok(cids)
    }

    async fn wallet_default(&self) -> anyhow::Result<Address> {
        // refer to: https://lotus.filecoin.io/reference/lotus/wallet/#walletdefaultaddress
        let r = self
            .client
            .request::<String>(methods::WALLET_DEFAULT_ADDRESS, json!({}))
            .await?;
        log::debug!("received wallet_default response: {r:?}");

        let addr = Address::from_str(&r)?;
        Ok(addr)
    }

    async fn wallet_list(&self) -> anyhow::Result<WalletListResponse> {
        // refer to: https://lotus.filecoin.io/reference/lotus/wallet/#walletlist
        let r = self
            .client
            .request::<WalletListResponse>(methods::WALLET_LIST, json!({}))
            .await?;
        log::debug!("received wallet_list response: {r:?}");
        Ok(r)
    }

    async fn wallet_new(&self, key_type: WalletKeyType) -> anyhow::Result<String> {
        let key_type_str = key_type.as_ref();
        // refer to: https://lotus.filecoin.io/reference/lotus/wallet/#walletnew
        let r = self
            .client
            .request::<String>(methods::WALLET_NEW, json!([key_type_str]))
            .await?;
        log::debug!("received wallet_new response: {r:?}");
        Ok(r)
    }

    async fn read_state<State: DeserializeOwned + Debug>(
        &self,
        address: Address,
        tipset: Cid,
    ) -> anyhow::Result<ReadStateResponse<State>> {
        // refer to: https://lotus.filecoin.io/reference/lotus/state/#statereadstate
        let r = self
            .client
            .request::<ReadStateResponse<State>>(
                methods::STATE_READ_STATE,
                json!([address.to_string(), [CIDMap::from(tipset)]]),
            )
            .await?;
        log::debug!("received read_state response: {r:?}");
        Ok(r)
    }

    async fn chain_head(&self) -> anyhow::Result<ChainHeadResponse> {
        let r = self
            .client
            .request::<ChainHeadResponse>(methods::CHAIN_HEAD, NO_PARAMS)
            .await?;
        log::debug!("received chain_head response: {r:?}");
        Ok(r)
    }
}
