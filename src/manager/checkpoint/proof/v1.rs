use crate::lotus::LotusClient;
use anyhow::anyhow;
use cid::Cid;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct V1Proof {
    tip_set: Vec<Cid>,
    states: Vec<Cid>,
}

pub async fn create_proof<L: LotusClient>(
    client: &L,
    height: ChainEpoch,
) -> anyhow::Result<V1Proof> {
    let tip_sets = client.chain_head().await?.cids;
    if tip_sets.is_empty() {
        return Err(anyhow!("chain head has empty cid"));
    }

    let response = client
        .get_tipset_by_height(height, Cid::try_from(&tip_sets[0])?)
        .await?;
    Ok(V1Proof {
        tip_set: response.tip_set_cids()?,
        states: response.blocks_state_roots()?,
    })
}
