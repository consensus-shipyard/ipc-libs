//! The macro to use when register the handler.

/// This macro is used to register the RPC handler to the RPC node. This macro contains the
/// node struct declaration and the node implementation is hidden from the caller so that we
/// can enforce a consistent calling pattern.
///
/// All handlers and their methods are registered under:
///     POST /json_rpc
/// One can attach the json payload body: `types::JSONRPCParam`.
///
/// As RPC handlers might have states, such as a db connection pool or a counter of requests served.
/// In this case, this macro is divided into two parts, one is the initialization of the handlers and
/// the other is the association with the method names. See below for an example.
///
/// # Examples
///
/// ```no_run
/// use agent::node::ClientNodeConfig;
///
/// agent::create_json_rpc_server!({
///     use agent::node::RPCNodeHandler;
///     use async_trait::async_trait;
///
///     pub struct Foo {}
///
///     #[async_trait]
///     impl RPCNodeHandler for Foo {
///         type Request = ();
///         type Output = String;
///         type Error = String;
///
///         async fn handle(&self, _request: &Self::Request) -> Result<Self::Output, Self::Error> {
///             Ok(String::from("foo"))
///         }
///     }
///
///     pub struct Bar {}
///
///     #[async_trait]
///     impl RPCNodeHandler for Bar {
///         type Request = ();
///         type Output = String;
///         type Error = String;
///
///         async fn handle(&self, _request: &Self::Request) -> Result<Self::Output, Self::Error> {
///             Ok(String::from("bar"))
///         }
///     }
///
///     let bar = Bar {};
///     let foo = Foo {};
///
///     agent::associate!(("bar", Bar, bar), ("foo", Foo, foo))
/// });
///
/// #[tokio::main]
/// async fn main() {
///     let config = ClientNodeConfig::default();
///     let rpc_node = node::IPCClientNode::new(config);
///
///     // Run on command line:
///     // curl --location --request POST 'http://localhost:3030/json_rpc' \
///     // --header 'Content-Type: application/json' \
///     // --data-raw '{
///     //     "id": 1,
///     //     "method": "bar",
///     //     "params": null,
///     //     "jsonrpc": "2.0"
///     // }'
///     // It should print:
///     //   {
///     //     "id": 1,
///     //     "jsonrpc": "2.0",
///     //     "result": "bar"
///     //   }
///     rpc_node.run().await;
/// }
/// ```
#[macro_export]
macro_rules! create_json_rpc_server {
    ( $init:block) => {
        mod node {
            use std::convert::Infallible;
            use warp::Filter;

            use $crate::node::config::ClientNodeConfig;
            use $crate::node::types::JSONRPCParam;
            use $crate::node::types::JSONRPCResponse;

            pub struct IPCClientNode {
                config: ClientNodeConfig,
            }

            impl IPCClientNode {
                pub fn new(config: ClientNodeConfig) -> Self {
                    Self { config }
                }

                /// Runs the node in the current thread
                pub async fn run(&self) {
                    let f = |bytes: bytes::Bytes| async move {
                        log::debug!("received bytes = {:?}", bytes);
                        let t = $init;

                        match serde_json::from_slice::<JSONRPCParam>(bytes.as_ref()) {
                            Ok(p) => {
                                let JSONRPCParam {
                                    id,
                                    method,
                                    params,
                                    jsonrpc,
                                } = p;
                                if let Some(handler) = t.get(method.as_str()) {
                                    match handler.handle(params).await {
                                        Ok(j) => {
                                            return Result::<_, Infallible>::Ok(warp::reply::json(
                                                &JSONRPCResponse {
                                                    id,
                                                    jsonrpc,
                                                    result: j,
                                                },
                                            ));
                                        }
                                        Err(s) => {
                                            log::error!("handler cannot process due to {s:?}");
                                            return Result::<_, Infallible>::Ok(warp::reply::json(
                                                &JSONRPCResponse {
                                                    id,
                                                    jsonrpc,
                                                    result: serde_json::Value::String(s),
                                                },
                                            ));
                                        }
                                    }
                                }
                                log::error!("method not supported {method:?}");
                                return Result::<_, Infallible>::Ok(warp::reply::json(
                                    &JSONRPCResponse {
                                        id,
                                        jsonrpc,
                                        result: serde_json::Value::String(format!(
                                            "Method {method} not supported"
                                        )),
                                    },
                                ));
                            }
                            Err(e) => {
                                log::error!("cannot parse parameter due to {e:?}");
                                return Result::<_, Infallible>::Ok(warp::reply::json(
                                    &JSONRPCResponse {
                                        id: 0,
                                        jsonrpc: String::from("2.0"),
                                        result: serde_json::Value::String(String::from(
                                            "Cannot parse parameters",
                                        )),
                                    },
                                ));
                            }
                        };
                    };

                    let json_rpc = warp::post()
                        .and(warp::path($crate::node::config::DEFAULT_RPC_ENDPOINT))
                        .and(warp::body::bytes())
                        .and_then(f);

                    log::info!("rpc node started at {:?}", self.config.addr());

                    warp::serve(json_rpc).run(self.config.addr()).await;
                }
            }
        }
    };
}

/// Associate the instance of the RPC handler to method name.
/// First entry is the method name, second is the type of the instance, third is the instance itself.
///
/// Note that currently we need to pass in `$type`, as we cannot derive the type of the instance handler
/// in this macro.
///
/// TODO: maybe we can use Derive procedure macro, can simplify the syntax a little.
///
/// # Examples
/// This macro takes comma separated tuple. Each tuple represents one association.
///
/// ```ignore
/// agent::associate!(
///     // first entry `"bar"` is the json rpc method name.
///     // second entry `Bar` is the type of the handle
///     // third entry `bar` is the instance
///     ("bar", Bar, bar),
///     // first entry `"foo"` is the json rpc method name.
///     // second entry `Foo` is the type of the handle
///     // third entry `foo` is the instance
///     ("foo", Foo, foo)
/// );
/// ```
#[macro_export]
macro_rules! associate {
    ( $(($method_name:expr, $type:ident, $handler:tt)),*) => {
        {
        enum Handlers {
            $($type($type),)*
        }

        impl Handlers {
            pub async fn handle(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
                use $crate::node::RPCNodeHandler;

                match self {
                    $(
                    Handlers::$type(handler) => {
                        match serde_json::from_value(params) {
                            Ok(v) => {
                                handler.handle(&v).await.map(|res| serde_json::to_value(res).unwrap())
                            },
                            Err(e) => {
                                log::error!("cannot parse parameter due to {e:?}");
                                return Err(String::from("Cannot parse parameters"));
                            }
                        }
                    }
                    )*
                }
            }
        }

        let mut handlers = std::collections::HashMap::new();
        $(
        handlers.insert($method_name, Handlers::$type($handler));
        )*

        handlers
        }
    }
}

#[cfg(test)]
mod test {
    use crate::node::config::{DEFAULT_NODE_ADDR, DEFAULT_PROTOCOL, DEFAULT_RPC_ENDPOINT};
    use crate::node::ClientNodeConfig;
    use std::thread::sleep;
    use std::time::Duration;

    crate::create_json_rpc_server!({
        use crate::node::RPCNodeHandler;
        use async_trait::async_trait;

        pub struct Foo {}

        #[async_trait]
        impl RPCNodeHandler for Foo {
            type Request = ();
            type Output = String;
            type Error = String;

            async fn handle(&self, _request: &Self::Request) -> Result<Self::Output, Self::Error> {
                Ok(String::from("foo"))
            }
        }

        pub struct Bar {}

        #[async_trait]
        impl RPCNodeHandler for Bar {
            type Request = ();
            type Output = String;
            type Error = String;

            async fn handle(&self, _request: &Self::Request) -> Result<Self::Output, Self::Error> {
                Ok(String::from("bar"))
            }
        }

        let bar = Bar {};
        let foo = Foo {};

        crate::associate!(("bar", Bar, bar), ("foo", Foo, foo))
    });

    #[tokio::test]
    async fn test_node() {
        let config = ClientNodeConfig::default();
        let rpc_node = node::IPCClientNode::new(config);

        let h = tokio::spawn(async move {
            rpc_node.run().await;
        });

        sleep(Duration::new(1, 0));

        tokio::spawn(async {
            let client = reqwest::Client::new();
            let res = client
                .post(format!(
                    "{}://{}/{}",
                    DEFAULT_PROTOCOL, DEFAULT_NODE_ADDR, DEFAULT_RPC_ENDPOINT
                ))
                .body("the exact body that is sent")
                .send()
                .await;
            println!("{res:?}");
        });

        sleep(Duration::new(2, 0));
        h.abort();
    }
}
