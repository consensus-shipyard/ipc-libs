use cid::Cid;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::MethodNum;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};

/// Init actor Exec Params, see https://github.com/filecoin-project/builtin-actors/blob/master/actors/init/src/types.rs#L17
#[derive(Serialize_tuple, Deserialize_tuple, Debug)]
pub(crate) struct ExecParams {
    pub code_cid: Cid,
    pub constructor_params: RawBytes,
}

/// Init actor exec method number, see https://github.com/filecoin-project/builtin-actors/blob/fb759f87fcd5de0a98cb61966cd27f680df83364/actors/init/src/lib.rs#L32
pub(crate) const INIT_EXEC_METHOD_NUM: MethodNum = 2;
