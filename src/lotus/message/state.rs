use serde::Deserialize;
use crate::lotus::message::CIDMap;

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
pub struct ReadStateResponse<State> {
    #[allow(dead_code)]
    pub balance: String,
    #[allow(dead_code)]
    pub code: CIDMap,
    #[allow(dead_code)]
    pub state: State,
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
