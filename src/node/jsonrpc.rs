use crate::node::config::{IPCAgentJsonRPCNodeConfig, DEFAULT_JSON_RPC_ENDPOINT};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::reply::with_status;
use warp::{Filter, Rejection, Reply};

/// The json rpc request param. It is the standard form our json-rpc and follows a structure similar
/// to the one of the Ethereum RPC: https://ethereum.org/en/developers/docs/apis/json-rpc/#curl-examples
#[derive(Serialize, Deserialize, Debug)]
struct JSONRPCRequest {
    pub id: u16,
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}

/// The json rpc response. It is the standard form our json-rpc and follows a structure similar
/// to the one of the Ethereum RPC: https://ethereum.org/en/developers/docs/apis/json-rpc/#curl-examples
#[derive(Debug, Serialize, Deserialize)]
struct JSONRPCResponse<T: Serialize> {
    pub id: u16,
    pub jsonrpc: String,
    pub result: T,
}

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
        log::info!("IPC agent rpc node listening at {:?}", self.config.addr());
        warp::serve(json_rpc_filter()).run(self.config.addr()).await;
    }
}

// Internal implementations

/// Create the json_rpc filter. The filter does the following:
/// - Listen to POST requests on the DEFAULT_JSON_RPC_ENDPOINT
/// - Extract the body of the request.
/// - Pass it to to the process function.
fn json_rpc_filter() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Copy {
    warp::post()
        .and(warp::path(DEFAULT_JSON_RPC_ENDPOINT))
        .and(warp::body::bytes())
        .and_then(to_json_rpc_request)
        .and_then(handle_request)
        .recover(handle_rejection)
}

async fn to_json_rpc_request(bytes: Bytes) -> Result<JSONRPCRequest, warp::Rejection> {
    serde_json::from_slice::<JSONRPCRequest>(bytes.as_ref()).map_err(|e| {
        log::debug!("cannot deserialize {bytes:?} due to {e:?}");
        warp::reject::custom(InvalidParameter)
    })
}

/// To handle the json rpc request. Currently just log it.
async fn handle_request(json_rpc_request: JSONRPCRequest) -> Result<impl Reply, warp::Rejection> {
    log::debug!("received json rpc request = {:?}", json_rpc_request);

    let JSONRPCRequest {
        id,
        method,
        params,
        jsonrpc,
    } = json_rpc_request;

    log::info!("received method = {method:?} and params = {params:?}");

    Ok(warp::reply::json(&JSONRPCResponse {
        id,
        jsonrpc,
        result: (),
    }))
}

/// The invalid parameter warp rejection error handling
#[derive(Debug)]
struct InvalidParameter;

impl Reject for InvalidParameter {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, warp::Rejection> {
    if err.is_not_found() {
        Ok(with_status("NOT_FOUND", StatusCode::NOT_FOUND))
    } else if err.find::<InvalidParameter>().is_some() {
        Ok(with_status("BAD_REQUEST", StatusCode::BAD_REQUEST))
    } else {
        log::error!("unhandled rejection: {:?}", err);
        Ok(with_status(
            "INTERNAL_SERVER_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::node::config::{DEFAULT_JSON_RPC_ENDPOINT, DEFAULT_JSON_RPC_VERSION};
    use crate::node::jsonrpc::{json_rpc_filter, JSONRPCRequest, JSONRPCResponse};
    use warp::http::StatusCode;

    #[tokio::test]
    async fn test_json_rpc_filter_works() {
        let filter = json_rpc_filter();

        let foo = "foo".to_string();
        let jsonrpc = String::from(DEFAULT_JSON_RPC_VERSION);
        let id = 0;

        let req = JSONRPCRequest {
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

        assert_eq!(StatusCode::BAD_REQUEST, value.status());
    }
}
