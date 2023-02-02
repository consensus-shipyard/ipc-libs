use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

const CID_ROOT_KEY: &str = "/";

pub type CIDResponse = HashMap<String, String>;

#[derive(Deserialize)]
pub struct MpoolPushMessageResponse {
    pub to: Address,
    pub from: Address,
    pub value: TokenAmount,
    pub method: MethodNum,
    pub params: Vec<u8>,

    pub nonce: Option<u64>,
    pub gas_limit: Option<TokenAmount>,
    pub gas_fee_cap: Option<TokenAmount>,
    pub gas_premium: Option<TokenAmount>,
    pub version: Option<u16>,
    pub max_fee: Option<TokenAmount>,

    pub cid: CIDResponse,
}

impl MpoolPushMessageResponse {
    pub fn get_root_cid(&self) -> Option<Cid> {
        self.cid
            .get(CID_ROOT_KEY)
            .map(|s| Cid::from_str(s).expect("server sent invalid cid"))
    }
}

pub struct MpoolPushMessage {
    pub to: Address,
    pub from: Address,
    pub value: TokenAmount,
    pub method: MethodNum,
    pub params: Vec<u8>,

    pub nonce: Option<u64>,
    pub gas_limit: Option<TokenAmount>,
    pub gas_fee_cap: Option<TokenAmount>,
    pub gas_premium: Option<TokenAmount>,
    pub cid: Option<Cid>,
    pub version: Option<u16>,
    pub max_fee: Option<TokenAmount>,
}

impl MpoolPushMessage {
    pub fn zero_value(to: Address, from: Address, method: MethodNum, params: Vec<u8>) -> Self {
        MpoolPushMessage {
            to,
            from,
            method,
            params,
            value: TokenAmount::from_atto(0),
            nonce: None,
            gas_limit: None,
            gas_fee_cap: None,
            gas_premium: None,
            cid: None,
            version: None,
            max_fee: None,
        }
    }
}
