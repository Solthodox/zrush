use crate::{
    node::node_proto::node_proto,
    utils::files::{read_from_file, write_to_file},
};
use node_proto::{node_client::NodeClient, AddBlockRequest, NodeInfoRequest, TransactionRequest};
use std::mem::forget;
use std::net::SocketAddr;
use tonic::Request;

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

pub async fn propagate_transaction(req: TransactionRequest) -> Result<(), ()> {
    let nodes = read_from_file("data/", "node_data.json").unwrap();
    let nodes: Vec<String> = serde_json::from_str(&nodes).unwrap();
    for node in nodes.iter() {
        let mut client = NodeClient::connect(node.clone()).await.unwrap();
        forget(
            client
                .request_send_transaction(Request::new(req.clone()))
                .await,
        );
    }
    Ok(())
}

pub async fn propagate_block(req: AddBlockRequest) -> Result<(), ()> {
    let nodes = read_from_file("data/", "node_data.json").unwrap();
    let nodes: Vec<String> = serde_json::from_str(&nodes).unwrap();
    for node in nodes.iter() {
        let mut client = NodeClient::connect(node.clone()).await.unwrap();
        forget(client.request_add_block(Request::new(req.clone())).await);
    }
    Ok(())
}
