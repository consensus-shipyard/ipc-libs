use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExecParams {
    pub code_cid: Cid,
    pub constructor_params: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CIDMap {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "/")]
    pub cid: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StateWaitMsgResponse {
    #[allow(dead_code)]
    message: CIDMap,
    #[allow(dead_code)]
    receipt: Receipt,
    #[allow(dead_code)]
    tip_set: Vec<CIDMap>,
    #[allow(dead_code)]
    height: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Receipt {
    #[allow(dead_code)]
    exit_code: u32,
    #[allow(dead_code)]
    r#return: String,
    #[allow(dead_code)]
    gas_used: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MpoolPushMessageResponse {
    pub to: Address,
    pub from: Address,
    pub value: TokenAmount,
    pub method: MethodNum,
    pub params: Vec<u8>,

    pub nonce: u64,
    pub gas_limit: Option<TokenAmount>,
    pub gas_fee_cap: Option<TokenAmount>,
    pub gas_premium: Option<TokenAmount>,
    pub version: Option<u16>,
    pub max_fee: Option<TokenAmount>,

    pub cid: CIDMap,
}

impl MpoolPushMessageResponse {
    pub fn get_root_cid(&self) -> Option<Cid> {
        self.cid
            .cid
            .as_ref()
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
    pub fn new(to: Address, from: Address, method: MethodNum, params: Vec<u8>) -> Self {
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

impl From<CIDMap> for Option<Cid> {
    fn from(m: CIDMap) -> Self {
        m.cid
            .map(|cid| Cid::from_str(&cid).expect("invalid cid str"))
    }
}

impl From<Option<Cid>> for CIDMap {
    fn from(c: Option<Cid>) -> Self {
        c.map(|cid| CIDMap {
            cid: Some(cid.to_string()),
        })
        .unwrap_or(CIDMap { cid: None })
    }
}

impl From<Cid> for CIDMap {
    fn from(c: Cid) -> Self {
        CIDMap {
            cid: Some(c.to_string()),
        }
    }
}
