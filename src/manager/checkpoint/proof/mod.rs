// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The checkpoint proof structs

mod v1;

pub use crate::manager::checkpoint::proof::v1::create_proof;
pub use crate::manager::checkpoint::proof::v1::V1Proof;
use serde::{Deserialize, Serialize};

/// The different version of checkpoint proofs
#[derive(Serialize, Deserialize, Debug)]
pub enum CheckpointProof {
    V1(V1Proof),
}
