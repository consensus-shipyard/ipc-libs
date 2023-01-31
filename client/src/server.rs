#[macro_export]
macro_rules! register_server_routes {
    ( $({$method:expr, $handler:ident, $struct:ident}), *) => {
        mod node {
            use std::convert::Infallible;
            use std::sync::Arc;
            use paste::paste;
            use async_trait::async_trait;
            use clap::Args;
            use serde::de::DeserializeOwned;
            use serde::{Deserialize, Serialize};
            use warp::Filter;

            use $crate::RPCNodeHandler;
            use $crate::CommandLineHandler;
            use $crate::ClientNodeConfig;
            use $crate::JSONRPCResponse;
            use $crate::JSONRPCParam;

            async fn process(bytes: bytes::Bytes) -> Result<impl warp::Reply, Infallible> {
                log::debug!("received bytes = {:?}", bytes);

                let r = match serde_json::from_slice::<JSONRPCParam>(bytes.as_ref()) {
                    Ok(p) => {
                        let JSONRPCParam { id, method, params, jsonrpc, .. } = p;
                        match method.as_str() {
                            $(
                            $method => match serde_json::from_value::<<super::$struct as RPCNodeHandler>::Request>(params) {
                                Ok(v) => match super::$handler.handle(&v).await {
                                    Ok(res) => {
                                        let j = serde_json::to_value(res).unwrap();
                                        warp::reply::json(&JSONRPCResponse {
                                            id: id,
                                            jsonrpc: jsonrpc,
                                            result: j
                                        })
                                    },
                                    Err(e) => {
                                        log::error!("handler cannot process due to {e:?}");
                                        warp::reply::json(&JSONRPCResponse {
                                            id,
                                            jsonrpc,
                                            result: serde_json::Value::String(String::from("Failed due to {e:}"))
                                        })
                                    }
                                },
                                Err(e) => {
                                    log::error!("cannot parse parameter due to {e:?}");
                                    warp::reply::json(&JSONRPCResponse {
                                        id,
                                        jsonrpc,
                                        result: serde_json::Value::String(String::from("Cannot parse parameters"))
                                    })
                                }
                            },
                            )*
                            _ => {
                                log::error!("method not supported {method:?}");
                                warp::reply::json(&JSONRPCResponse {
                                    id,
                                    jsonrpc,
                                    result: serde_json::Value::String(format!("Method {method} not supported"))
                                })
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("cannot parse parameter due to {e:?}");
                        warp::reply::json(&JSONRPCResponse {
                            id: 0,
                            jsonrpc: String::from("2.0"),
                            result: serde_json::Value::String(String::from("Cannot parse parameters"))
                        })
                    }
                };
                Ok(r)
            }

            pub struct IPCClientNode {
                config: ClientNodeConfig,
            }

            impl IPCClientNode {
                pub fn new(config: ClientNodeConfig) -> Self {
                    Self { config }
                }

                /// Runs the node in the current thread
                pub async fn run(&self) {
                    let json_rpc = warp::post()
                        .and(warp::path("json_rpc"))
                        .and(warp::body::bytes())
                        .and_then(process);

                    warp::serve(json_rpc).run(self.config.addr()).await;
                }
            }

            /// The config struct used parsed from cli
            #[derive(Deserialize, Debug, Default, Args)]
            #[command(about = "Launches the IPC node")]
            pub struct NodeLaunch {
                #[arg(
                    long = "config",
                    value_name = "CONFIG_FILE_PATH",
                    help = "The config file path for the IPC client node",
                    env = "IPC_CLIENT_NODE_CONFIG"
                )]
                config_path: Option<String>,
            }

            impl NodeLaunch {
                pub fn client_node_config(&self) -> ClientNodeConfig {
                    self.config_path
                        .as_ref()
                        .map(|s| parse_yaml(s))
                        .unwrap_or_default()
                }
            }

            pub struct NodeHandler {}

            #[async_trait]
            impl CommandLineHandler for NodeHandler {
                type Request = NodeLaunch;
                type Error = ();

                async fn handle(request: &Self::Request) -> Result<(), Self::Error> {
                    let node_config = request.client_node_config();
                    IPCClientNode::new(node_config).run().await;
                    Ok(())
                }
            }

            fn parse_yaml<T: DeserializeOwned>(path: &str) -> T {
                let raw = std::fs::read_to_string(path).expect("cannot read config yaml");
                serde_yaml::from_str(&raw).expect("cannot parse yaml")
            }
        }
    }
}
