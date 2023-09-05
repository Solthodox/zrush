use std::{
    fs::File,
    io::{stdin, Write},
    path::Path,
};

use crate::utils::files::{read_from_file, write_to_file};
use chrono::{Duration, Utc};
use ethers::types::{Address, Signature, U256};
use serde_derive::Serialize;
use tonic::{transport::Server, Code, Request, Response, Status};

mod node {
    include!("../grpc/node.rs");
}

use node::{
    node_client::NodeClient,
    node_server::{Node, NodeServer},
    RequestSyncResponse, SyncRequest, TransactionRequest, TransactionResponse,
};

use crate::block::block::Block;
use crate::{signature::verification::verify_signature, transaction::core::Transaction};

#[derive(Debug)]
pub enum NodeError {
    SyncError(String),
    NetworkError(String),
    InvalidConfigInput(String),
}

#[derive(Default)]
pub struct NodeService;

#[tonic::async_trait]
impl Node for NodeService {
    async fn request_send_transaction(
        &self,
        tx: Request<TransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let tx = tx.into_inner();
        let tx = parse_grpc_transaction_request(tx)
            .map_err(|err| Status::new(Code::InvalidArgument, err))?;
        let msg = format!(
            "zrush_signed_message:{}{}{:?}",
            tx.from(),
            tx.to(),
            tx.amount()
        );
        match verify_signature(&msg, tx.signature(), tx.from()) {
            true => return Ok(Response::new(TransactionResponse {})),
            false => return Err(Status::new(Code::InvalidArgument, "Invalid signature")),
        }
    }

    async fn request_sync(
        &self,
        req: Request<SyncRequest>,
    ) -> Result<Response<RequestSyncResponse>, Status> {
        let config = read_from_file("data/", "chain_config.json");
        let data = read_from_file("data/", "storage/chain_data.json");
        if let (Ok(config_content), Ok(data_content)) = (config, data) {
            return Ok(Response::new(RequestSyncResponse {
                network_settings: config_content,
                data: data_content,
            }));
        }
        return Err(Status::new(Code::DataLoss, "Error reading data"));
    }
}

fn parse_grpc_transaction_request(tx: TransactionRequest) -> Result<Transaction, &'static str> {
    let from: [u8; 20] = tx.from.try_into().map_err(|_| "Invalid from")?;
    let to: [u8; 20] = tx.to.try_into().map_err(|_| "Invalid to")?;
    let amount: [u8; 32] = tx.amount.try_into().map_err(|_| "Invalid amount")?;
    let fee: [u8; 32] = tx.fee.try_into().map_err(|_| "Invalid fee")?;
    let signature = tx.signature.unwrap();
    let r: [u8; 32] = signature.r.try_into().map_err(|_| "Invalid r")?;
    let s: [u8; 32] = signature.s.try_into().map_err(|_| "Invalid s")?;
    let v = signature.v;

    let from = Address::from(from);
    let to = Address::from(to);
    let amount = U256::from(amount);
    let fee = U256::from(fee);
    let r = U256::from(r);
    let s = U256::from(s);

    Ok(Transaction::new(
        from,
        to,
        amount,
        fee,
        Signature { r, s, v },
    ))
}

#[derive(Debug, Default, Serialize, Clone)]

pub struct ChainConfig {
    name: String,
    chain_id: u8,
    initial_block_reward: U256,
    creation_timestamp: u64,
    seconds_between_blocks: u8,
    months_between_halvings: u64,
}
#[derive(Debug, Default, Serialize, Clone)]

pub struct NodeConfig {
    wallet_address: Address,
}

pub fn create_new_blockchain() -> Result<(), NodeError> {
    println!(
        "
███████╗██████╗ ██╗   ██╗███████╗██╗  ██╗
╚══███╔╝██╔══██╗██║   ██║██╔════╝██║  ██║
  ███╔╝ ██████╔╝██║   ██║███████╗███████║
 ███╔╝  ██╔══██╗██║   ██║╚════██║██╔══██║
███████╗██║  ██║╚██████╔╝███████║██║  ██║
╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝                                         
    "
    );
    println!(
        "Welcome to zrush v1.1
    
    "
    );
    let (chain_config, node_config) = config_blockchain()
        .map_err(|err_msg| NodeError::InvalidConfigInput(err_msg.to_string()))?;
    let first_block = Block::genesis_block(
        &chain_config.initial_block_reward,
        &node_config.wallet_address,
    );
    let serialization = serde_json::to_string(&first_block).map_err(|_| {
        NodeError::InvalidConfigInput(String::from("Failed to serialize genesis block"))
    })?;
    let _ = write_to_file("./data/storage", "chain_data.json", &serialization);
    Ok(())
}

fn config_blockchain() -> Result<(ChainConfig, NodeConfig), &'static str> {
    println!("Name:");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .map_err(|_| "stdin: Failed to read name")?;
    let name = buf
        .trim()
        .parse::<String>()
        .map_err(|_| "Invalid name")?
        .to_lowercase();

    println!("Chain id:");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .map_err(|_| "stdin: Failed to read chain id")?;
    let chain_id = buf.trim().parse::<i32>().map_err(|_| "Invalid chain id")? as u8;

    println!("Seconds between blocks:");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .map_err(|_| "stdin: Failed to read seconds between blocks")?;
    let seconds_between_blocks = buf
        .trim()
        .parse::<u8>()
        .map_err(|_| "Invalid seconds between blocks")?;

    println!("Initial block reward: ");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .map_err(|_| "stdin: Failed to read initial block reward")?;
    let initial_block_reward = buf
        .trim()
        .parse::<String>()
        .map_err(|_| "Invalid read initial block reward")?;
    let mut target = [0u8; 32];
    let source = initial_block_reward.as_bytes();
    let fill_bytes = |target: &mut [u8], source: &[u8], bytes: usize| {
        for (target_elem, source_elem) in target.iter_mut().zip(source.iter().take(bytes)) {
            *target_elem = *source_elem;
        }
    };
    fill_bytes(&mut target, &source, 32);

    let initial_block_reward = U256::from(&target);
    println!("Months between halvings: ");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .map_err(|_| "stdin: Failed to read months between halving")?;
    let months_between_halvings = buf
        .trim()
        .parse::<i64>()
        .map_err(|_| "Invalid months between halving")?;
    let months_between_halvings = months_to_milliseconds(months_between_halvings) as u64;

    println!("Wallet address: ");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .map_err(|_| "stdin: Failed to read wallet address");
    let wallet_address: String = buf.trim().parse().map_err(|_| "Failed")?;
    let mut target = [0u8; 20];
    let source = wallet_address.as_bytes();

    fill_bytes(&mut target, &source, 20);
    let node_config = NodeConfig {
        wallet_address: Address::from(&target),
    };

    let creation_timestamp = Utc::now().timestamp_millis() as u64;

    let chain_config = ChainConfig {
        name: name.clone(),
        chain_id,
        initial_block_reward,
        creation_timestamp,
        seconds_between_blocks,
        months_between_halvings,
    };

    let content =
        serde_json::to_string(&chain_config).map_err(|_| "Failed to serialize chain config")?;
    write_to_file("./data", "chain_config.json", &content).unwrap();
    println!("Chain config saved to : data/chain_config.json");
    Ok((chain_config, node_config))
}

fn months_to_milliseconds(months: i64) -> i64 {
    let current_time = Utc::now();
    let target_time = current_time
        .checked_add_signed(Duration::days(months * 30))
        .unwrap();
    let target_naive = target_time.naive_utc();
    let difference = target_naive.timestamp_millis() - current_time.timestamp_millis();

    difference
}

pub async fn sync_node(boot_node_addr: String) -> Result<(), NodeError> {
    println!("Syncing node...");
    let mut client = NodeClient::connect(boot_node_addr.clone())
        .await
        .expect("Failed to connect to the boot node");
    let sync_request = SyncRequest {};
    let res = client.request_sync(sync_request).await;

    if let Ok(sync_response) = res {
        let res = sync_response.into_inner();
        println!("{:#?}", { res.clone() });
        write_to_file("./data/storage", "chain_data.json", &res.data).unwrap();
        write_to_file("./data/", "chain_config.json", &res.network_settings).unwrap();
        println!("Node successfully connected.");
        return Ok(());
    } else {
        return Err(NodeError::SyncError(format!(
            "Could not sync with: {boot_node_addr}"
        )));
    }
}

pub async fn run_node(port: String) -> Result<(), NodeError> {
    let addr = String::from("127.0.0.1:") + port.as_str();
    println!("✔️ Running node in: {addr}");
    let parsed_addr = addr
        .parse()
        .map_err(|_| NodeError::InvalidConfigInput(String::from("Invalid port")))?;

    let node_service = NodeService::default();
    Server::builder()
        .add_service(NodeServer::new(node_service))
        .serve(parsed_addr)
        .await
        .map_err(|err| NodeError::NetworkError(err.to_string()))?;
    Ok(())
}
