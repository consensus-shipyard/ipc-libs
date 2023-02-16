use crate::node::config::{
    IPCAgentJsonRPCNodeConfig, DEFAULT_JSON_RPC_ENDPOINT, DEFAULT_JSON_RPC_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use warp::Filter;

/// The IPC JSON RPC node that contains all the methods and handlers. The underlying implementation
/// is using `warp`.
///
/// Note that currently only http json rpc is supported.
///
/// # Examples
/// ```no_run
/// use agent::node::config::IPCAgentJsonRPCNodeConfig;
/// use agent::node::jsonrpc::IPCAgentJsonRPCNode;
///
/// #[tokio::main]
/// async fn main() {
///     let n = IPCAgentJsonRPCNode::new(IPCAgentJsonRPCNodeConfig::default());
///     n.run().await;
/// }
/// ```
pub struct IPCAgentJsonRPCNode {
    config: IPCAgentJsonRPCNodeConfig,
}

impl IPCAgentJsonRPCNode {
    pub fn new(config: IPCAgentJsonRPCNodeConfig) -> Self {
        Self { config }
    }

    /// Runs the node in the current thread
    pub async fn run(&self) {
        log::info!("rpc node started at {:?}", self.config.addr());
        warp::serve(json_rpc_filter()).run(self.config.addr()).await;
    }
}

// Internal implementations

/// Create the json_rpc filter
fn json_rpc_filter() -> impl Filter<Extract = (warp::reply::Json,), Error = warp::Rejection> + Copy
{
    warp::post()
        .and(warp::path(DEFAULT_JSON_RPC_ENDPOINT))
        .and(warp::body::bytes())
        .and_then(process)
}

/// The process method for the JSON RPC node.
async fn process(bytes: bytes::Bytes) -> Result<warp::reply::Json, Infallible> {
    log::debug!("received bytes = {:?}", bytes);

    match serde_json::from_slice::<JSONRPCParam>(bytes.as_ref()) {
        Ok(p) => {
            let JSONRPCParam {
                id,
                method,
                params,
                jsonrpc,
            } = p;

            log::debug!("received method = {method:?} and params = {params:?}");

            Ok(warp::reply::json(&JSONRPCResponse {
                id,
                jsonrpc,
                result: (),
            }))
        }
        Err(e) => {
            log::error!("cannot parse parameter due to {e:?}");
            Ok(warp::reply::json(&JSONRPCResponse {
                id: 0,
                jsonrpc: String::from(DEFAULT_JSON_RPC_VERSION),
                result: serde_json::Value::String(String::from("Cannot parse parameters")),
            }))
        }
    }
}

/// Follows: https://ethereum.org/en/developers/docs/apis/json-rpc/#curl-examples
#[derive(Serialize, Deserialize)]
struct JSONRPCParam {
    pub id: u16,
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JSONRPCResponse<T: Serialize> {
    pub id: u16,
    pub jsonrpc: String,
    pub result: T,
}

#[cfg(test)]
mod test {
    use crate::node::config::{DEFAULT_JSON_RPC_ENDPOINT, DEFAULT_JSON_RPC_VERSION};
    use crate::node::jsonrpc::{json_rpc_filter, JSONRPCParam, JSONRPCResponse};

    #[tokio::test]
    async fn test_json_rpc_filter_works() {
        let filter = json_rpc_filter();

        let foo = "foo".to_string();
        let jsonrpc = String::from(DEFAULT_JSON_RPC_VERSION);
        let id = 0;

        let req = JSONRPCParam {
            id,
            jsonrpc: jsonrpc.clone(),
            method: foo.clone(),
            params: Default::default(),
        };
        // Execute `sum` and get the `Extract` back.
        let value = warp::test::request()
            .method("POST")
            .path(&format!("/{DEFAULT_JSON_RPC_ENDPOINT:}"))
            .json(&req)
            .reply(&filter)
            .await;

        let v = serde_json::from_slice::<JSONRPCResponse<()>>(value.body()).unwrap();

        assert_eq!(v.id, id);
        assert_eq!(v.jsonrpc, jsonrpc);
        assert_eq!(v.result, ());
    }

    #[tokio::test]
    async fn test_json_rpc_filter_cannot_parse_param() {
        let filter = json_rpc_filter();

        // Execute `sum` and get the `Extract` back.
        let value = warp::test::request()
            .method("POST")
            .path(&format!("/{DEFAULT_JSON_RPC_ENDPOINT:}"))
            .json(&())
            .reply(&filter)
            .await;

        let v = serde_json::from_slice::<JSONRPCResponse<String>>(value.body()).unwrap();
        assert_eq!(v.result, String::from("Cannot parse parameters"));
    }
}
