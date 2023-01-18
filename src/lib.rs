use async_trait::async_trait;
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};

#[async_trait]
trait JsonrpcClient {
    // TODO: use an error type that doesn't leak implementation details
    async fn request(&self, method: &str) -> Result<String, reqwest::Error>;
}

pub struct ReqwestJsonrpcClient {
    client: Client,
    url: &'static str,
}

impl ReqwestJsonrpcClient {
    pub fn new(client: Client, url: &'static str) -> Self {
        Self { client, url }
    }
}

#[async_trait]
impl JsonrpcClient for ReqwestJsonrpcClient {
    async fn request(&self, method: &str) -> Result<String, reqwest::Error> {
        let request_body = format!("{{\"jsonrpc\": \"2.0\", \"id\":1, \"method\":\"{}\"}}", method);
        let response = self.client.post(self.url)
            .headers(HeaderMap::from_iter(
                [(CONTENT_TYPE, HeaderValue::from_static("application/json"))]))
            .body(request_body)
            .send()
            .await?;

        println!("{}", response.status());
        let response_body = response.text().await?;
        println!("{}", response_body);

        Ok(response_body)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[tokio::test]
    async fn test_jsonrpc() {
        let url = "https://api.calibration.node.glif.io/rpc/v0";
        let client = ReqwestJsonrpcClient::new(Client::new(), url);
        client.request("Filecoin.ChainHead").await.unwrap();
    }

    #[test]
    fn test_json_deserialization() {
        let json = r#"{"jsonrpc":"2.0","result":{"Cids":[{"/":"bafy2bzacea4tzhxzh53opjfam734offtlcx7ogg6vyzdkl2v5tyb76s2iivhu"}],"Blocks":[{"Miner":"t01491","Ticket":{"VRFProof":"oiHY0zp+SAlQu/lCXUhXAcgeDoH81b70DM2GsWkzbW1c2t7BqjU7NWBjTrbiXgtwEza4N699qOEhdkMiq6e/b5vWja0s3YmZ1qmtIlAfSQebxmXQhXB1cbhVNphb/6N/"},"ElectionProof":{"WinCount":1,"VRFProof":"teKGBALyOc80BH0/n4fL3cQ80nGFWah5mKjy+RIBmGiBnV/4OG8nwWZAQ4wz/6aUAio9L2HYXv51Gj1p9q/hHLEuD69BWRM3BDB0v5BwISOt21GdGoIle2KZEbjxlWSj"},"BeaconEntries":[{"Round":2620011,"Data":"tG4hyBZHRNfTZz8PSL3OAguWwO2xDwd1kM7mlYIy3uionm79CZj6zgf4vNVjECtCAVoHoyj4hp0IksMsUUiNVzqr61LGVnIXiMm80aJZdpedY0J/tBYCtl6ownYoCf9u"}],"WinPoStProof":[{"PoStProof":3,"ProofBytes":"p8w9aN0RdVycaY//qHRHYT4Qs0XZeso55pzdBmqIIvtNY7m+KdkSlW4H0v4DzAJ2k1mIH6f+pqoWuBbGgBo87bJiDvTkEd1RUzXdyZCjzy86wYU/abMMaz6ZBtRrX59sB7NORRKC6W/MH5RVgBGsrEGiMn9GNEFP+jP0YB8tFVTHGny8qjUOMixfMQUegKOFi+GjqHV0tZO9Eeww7WecEWzXkfR6ZEBbKY2ZhRFZCC6cGuQd5IRHPt/qRUu4wRj6"}],"Parents":[{"/":"bafy2bzacedkvtrt3umb3kjwuwoevfpv4gblkt52uq3gv4fqa3wmlwtqkzw5fm"},{"/":"bafy2bzacectyj6piz4u5iihhvxdgtwm6brfypix67fefb7764xcrwdsligaf4"},{"/":"bafy2bzaceah3rohz32nsf2cwqgfn4a2xy4u3vt7ubjaxap3j7qs7jo46begfu"}],"ParentWeight":"4198218457","Height":223500,"ParentStateRoot":{"/":"bafy2bzaceduvhux3qfr4zdcyhdwtukkj5cybj4lyauxp3e2hcuewxhcjgyzlu"},"ParentMessageReceipts":{"/":"bafy2bzaceat7x2o43ns5oagoxtgk2pnabka4sr3dv2wcfrhmsmroixe7irngw"},"Messages":{"/":"bafy2bzacecmda75ovposbdateg7eyhwij65zklgyijgcjwynlklmqazpwlhba"},"BLSAggregate":{"Type":2,"Data":"wAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"},"Timestamp":1674031380,"BlockSig":{"Type":2,"Data":"l7onUaHkjeZU+ryUUxK16V5FvwENLVZljawvj2LPE77P7N8+k2Mk2wtV8s3xgj4/AqTjZKHGGVuJrodxQjoo62yLi7eOuk1AX/wucj4wZc3q2+MbJqsNaJSl0kyeNhhD"},"ForkSignaling":0,"ParentBaseFee":"100"}],"Height":223500},"id":1}"#;
        let v: Value = serde_json::from_str(json).unwrap();
        println!("{}", v["id"]);
        println!("{}", v["result"]);
        println!("{}", v["result"]["Height"]);
    }
}
