use anyhow::{anyhow, Result};
use async_channel::{Receiver, Sender};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::json;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio_tungstenite::{connect_async, WebSocketStream};
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

#[cfg(test)]
mod tests;

/// A convenience constant that represents empty params in a JSON-RPC request.
pub const NO_PARAMS: Value = json!([]);

/// A simple async JSON-RPC client that can send one-shot request via HTTP/HTTPS
/// and subscribe to push-based notifications via Websockets. The returned
/// results are of type [`Value`] from the [`serde_json`] crate.
#[async_trait]
pub trait JsonRpcClient {
    /// Sends a JSON-RPC request with `method` and `params` via HTTP/HTTPS.
    async fn request(&self, method: &str, params: Value) -> Result<Value>;

    /// Subscribes to notifications via a Websocket. This returns a [`Receiver`]
    /// channel that is used to receive the messages sent by the server.
    async fn subscribe(&self, method: &str) -> Result<Receiver<Value>>;
}

/// The implementation of [`JsonRpcClient`].
pub struct JsonRpcClientImpl {
    http_client: Client,
    url: Url,
}

impl JsonRpcClientImpl {
    /// Creates a client that sends all requests to `url`.
    pub fn new(url: Url) -> Self {
        Self { http_client: Client::default(), url }
    }
}

#[async_trait]
impl JsonRpcClient for JsonRpcClientImpl {
    async fn request(&self, method: &str, params: Value) -> Result<Value> {
        let request_body = build_jsonrpc_request(method, params)?;
        let response = self.http_client.post(self.url.as_str())
            .headers(HeaderMap::from_iter(
                [(CONTENT_TYPE, HeaderValue::from_static("application/json"))]))
            .body(request_body)
            .send().await?;

        let response_body = response.text().await?;
        let value = serde_json::from_str(response_body.as_str())?;

        Ok(value)
    }

    async fn subscribe(&self, method: &str) -> Result<Receiver<Value>> {
        let (mut ws_stream, _) = connect_async(self.url.as_str()).await?;
        let request_body = build_jsonrpc_request(method, NO_PARAMS)?;
        ws_stream.send(Message::text(request_body)).await?;

        let (send_chan, recv_chan) = async_channel::unbounded::<Value>();
        spawn(handle_stream(ws_stream, send_chan));

        Ok(recv_chan)
    }
}

// Processes a websocket stream by reading messages from the stream `ws_stream` and sending
// them to an output channel `chan`.
async fn handle_stream(mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>, chan: Sender<Value>) {
    loop {
        match ws_stream.next().await {
            None => {
                // TODO: does this mean that the channel was closed?
                log::error!("No message in websocket stream");
                break;
            }
            Some(result) => {
                match result {
                    Ok(msg) => {
                        log::trace!("Read message from websocket stream: {}", msg);
                        let value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
                        chan.send(value).await.unwrap();
                    }
                    Err(err) => {
                        log::error!("Error reading message from websocket stream: {}", err);
                        break;
                    }
                }
            }
        };
    }
    chan.close();
}

// A convenience function to build and serialize a JSON-RPC request.
fn build_jsonrpc_request(method: &str, params: Value) -> Result<String> {
    let has_params =
        if params.is_array() {
            let array_params = params.as_array().unwrap();
            !array_params.is_empty()
        } else if params.is_object() {
            let object_params = params.as_object().unwrap();
            !object_params.is_empty()
        } else {
            return Err(anyhow!("params is not an array nor an object"));
        };


    let request_value =
        if has_params {
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params,
            })
        } else {
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
            })
        };
    Ok(request_value.to_string())
}
