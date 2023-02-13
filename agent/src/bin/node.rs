use agent::node::ClientNodeConfig;

agent::create_json_rpc_server!({
    use agent::node::RPCNodeHandler;
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

    let b = Bar {};
    let f = Foo {};

    agent::associate!(("bar", Bar, b), ("foo", Foo, f))
});

#[tokio::main]
async fn main() {
    let config = ClientNodeConfig::default();
    let rpc_node = node::IPCClientNode::new(config);

    // Run on command line:
    // curl --location --request POST 'http://localhost:3030/json_rpc' \
    // --header 'Content-Type: application/json' \
    // --data-raw '{
    //     "id": 1,
    //     "method": "bar",
    //     "params": null,
    //     "jsonrpc": "2.0"
    // }'
    // It should print:
    //   {
    //     "id": 1,
    //     "jsonrpc": "2.0",
    //     "result": "bar"
    //   }
    rpc_node.run().await;
}
