use std::net::SocketAddr;
use tonic::Request;
mod node {
    include!("../grpc/node.rs");
}

use node::{
    node_client::NodeClient,
    NodeInfoRequest
};
use crate::utils::files::{read_from_file, write_to_file};

pub async fn connect_node(client_address: Option<SocketAddr>) -> Result<(), ()> {
    if let Some(addr) = client_address {
        let addr: String = addr.ip().to_string();
        let mut client = NodeClient::connect(addr.clone()).await.unwrap();
        let res = client
            .request_node_info(Request::new(NodeInfoRequest {}))
            .await;

        if let Ok(response) = res {
            let nodes = read_from_file("data/", "node_data.json").unwrap();
            let mut nodes: Vec<String> = serde_json::from_str(&nodes).unwrap();
            let response = response.into_inner();
            let address = response.address;
            nodes.push(address);
            let content = serde_json::to_string(&nodes).unwrap();
            write_to_file("data/", "node_data.json", &content).unwrap();
        }
    }
    Ok(())
}
