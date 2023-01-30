#[macro_export]
macro_rules! register_server_routes {
    ( $({$method:ident, $path:expr, $handler:ident}), *) => {
            use std::convert::Infallible;
            use client::ClientNodeConfig;
            use std::sync::Arc;
            use warp::Filter;
            use client::RPCNodeHandler;
            use paste::paste;

            $(
            paste!{
                async fn [<$handler:snake>]() -> Result<impl warp::Reply, Infallible> {
                    Ok(warp::reply::json(&$handler.handle(&()).await.unwrap()))
                }
            }

            )*

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
                        .map(|bytes: bytes::Bytes| {
                            log::info!("received bytes = {:?}", bytes);
                            warp::reply::json(&())
                        });

                    let routes = json_rpc;
                    $(

                    paste!{
                        let routes = routes.or(
                            warp::$method()
                            .and(warp::path($path))
                            .and_then([<$handler:snake>])
                        );
                    }

                    )*

                    warp::serve(routes).run(self.config.addr()).await;
                }
            }

            // use $crate::CommandLineHandler;
            // use $crate::ClientNodeConfig;
            use async_trait::async_trait;
            use clap::Args;
            use serde::de::DeserializeOwned;
            use serde::Deserialize;

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
