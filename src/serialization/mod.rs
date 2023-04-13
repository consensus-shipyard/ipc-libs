// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
// Copyright 2022-2023 Protocol Labs
//! Handles the serialization of different types between actor cbor tuple serialization and json rpc
//! json serialization.

use std::ops::Deref;

mod checkpoint;
mod voting;

/// A helper struct to serialize struct to json.
///
/// Most of the types should have no need to use this struct. But some types that are shared between
/// actor, which are using cbor tuple serialization and json rpc response. We are using this wrapper
/// to handle convert to json instead.
#[derive(Debug)]
pub struct SerializeToJson<T>(pub T);

/// A helper struct to deserialize struct from json, rationale similar to the above.
#[derive(Debug)]
pub struct DeserializeFromJson<T>(T);

impl<T> DeserializeFromJson<T> {
    pub fn new(t: T) -> Self {
        Self(t)
    }
}

impl<T> Deref for DeserializeFromJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
