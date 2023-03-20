use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use num_traits::ToPrimitive;
use serde::Serializer;

pub fn serialize_subnet_id_to_str<S>(id: &SubnetID, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&id.to_string())
}

pub fn serialize_token_amount_to_atto<S>(amount: &TokenAmount, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u64(amount.atto().to_u64().unwrap_or(0))
}