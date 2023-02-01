#[macro_export]
macro_rules! register_server_routes {
    ( init: $init:block, commands: $($method:tt), *) => {
        mod node {
            use std::convert::Infallible;
            use std::sync::Arc;
            use paste::paste;
            use async_trait::async_trait;
            use clap::Args;
            use serde::de::DeserializeOwned;
            use serde::{Deserialize, Serialize};
            use warp::Filter;

            use $crate::common::handlers::RPCNodeHandler;
            use $crate::common::handlers::CommandLineHandler;
            use $crate::common::config::ClientNodeConfig;
            use $crate::common::rpc::JSONRPCResponse;
            use $crate::common::rpc::JSONRPCParam;

            async fn process(bytes: bytes::Bytes) -> Result<impl warp::Reply, Infallible> {
                log::debug!("received bytes = {:?}", bytes);

                let (
                    $($method,)*
                ) = $init;

                let r = match serde_json::from_slice::<JSONRPCParam>(bytes.as_ref()) {
                    Ok(p) => {
                        let JSONRPCParam { id, method, params, jsonrpc } = p;
                        match method.as_str() {
                            $(
                             stringify!($method) => match serde_json::from_value(params) {
                                Ok(v) => match $method.handle(&v).await {
                                    Ok(res) => {
                                        let j = serde_json::to_value(res).unwrap();
                                        warp::reply::json(&JSONRPCResponse {
                                            id,
                                            jsonrpc,
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
                            }
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
                    },
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

            pub struct NodeCmd {}

            #[async_trait]
            impl CommandLineHandler for NodeCmd {
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

#[macro_export]
macro_rules! register_cli_command {
    ( $({$name:ident, $handler:ty}), *) => {
        use clap::{Parser, Subcommand};
        use $crate::common::handlers::CommandLineHandler;

        /// The overall command line struct
        #[derive(std::fmt::Debug, Parser)]
        #[command(
            name = "ipc",
            about = "The IPC node command line tool",
            version = "v0.0.1"
        )]
        #[command(propagate_version = true)]
        struct IPCNode {
            #[command(subcommand)]
            command: Commands,
        }

        /// The subcommand to be called
        #[derive(Debug, Subcommand)]
        enum Commands {
            $(
                $name(<$handler as CommandLineHandler>::Request),
            )*

        }

        pub async fn cli() {
            let args = IPCNode::parse();
            let r = match &args.command {
            $(
                Commands::$name(n) => <$handler as CommandLineHandler>::handle(n).await,
            )*
            };

            if r.is_err() {
                println!(
                    "process command: {:?} failed due to {:?}",
                    args.command,
                    r.unwrap_err()
                )
            }
        }
    }
}
