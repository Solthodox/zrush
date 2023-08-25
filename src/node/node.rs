use chrono::{Duration, Utc};
use ethers::types::{Address, Signature, U256};
use serde_derive::Serialize;
use std::fs::File;

use std::io::{stdin, Read, Write};
use std::path::Path;
use tonic::{transport::Server, Code, Request, Response, Status};

mod node {
    include!("../grpc/node.rs");
}

use node::{
    node_server::{Node, NodeServer},
    RequestSyncResponse, SyncRequest, TransactionRequest, TransactionResponse,
};

use crate::block::block::Block;
use crate::{signature::verification::verify_signature, transaction::core::Transaction};

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
        let folder_path = "data/";
        let file_name = String::from("chain_config.json");
        let file_path = Path::new(folder_path).join(file_name);
        match File::open(file_path) {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                return Ok(Response::new(RequestSyncResponse {
                    network_settings: content,
                }));
            }
            Err(_) => return Err(Status::new(Code::NotFound, "Failed to sync")),
        }
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

pub fn create_new_blockchain() {
    println!("================================================");
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
    println!("================================================");
    let bytes_example: [u8; 20] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
    let example_address = Address::from(&bytes_example);
    println!("address : {:?}", example_address);
    let (chain_config, node_config) = config_blockchain();
    let first_block = Block::genesis_block(
        &chain_config.initial_block_reward,
        &node_config.wallet_address,
    );
    let serialization = serde_json::to_string(&first_block).unwrap();
    let folder_path = "data/storage";
    let file_name = String::from("chain.json");
    let file_name = file_name.as_str();
    std::fs::create_dir_all(folder_path).unwrap();
    let file_path = Path::new(folder_path).join(file_name);
    let mut file = File::create(file_path).unwrap();

    let content = serde_json::to_string(&serialization).expect("Failed to serialize chain config");
    file.write_all(content.as_bytes()).unwrap();

    println!("Data has been written to the file.");
    println!("chain config saved to : {file_name}");
}

fn config_blockchain() -> (ChainConfig, NodeConfig) {
    println!("Name:");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .expect("stdin: Failed to read line");
    let name = buf.trim().parse::<String>().unwrap().to_lowercase();

    println!("Chain id:");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .expect("stdin: Failed to read line");
    let chain_id = buf.trim().parse::<i32>().unwrap() as u8;

    println!("Seconds between blocks:");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .expect("stdin: Failed to read line");
    let seconds_between_blocks = buf.trim().parse::<u8>().unwrap();

    println!("Initial block reward: ");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .expect("stdin: Failed to read line");
    let initial_block_reward = buf.trim().parse::<String>().unwrap();
    let mut target = [0u8; 32];
    let source = initial_block_reward.as_bytes();

    for (target_elem, source_elem) in target.iter_mut().zip(source.iter().take(32)) {
        *target_elem = *source_elem;
    }

    let initial_block_reward = U256::from(&target);
    println!("Months between halvings: ");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .expect("stdin: Failed to read line");
    let months_between_halvings = buf.trim().parse::<i64>().unwrap();
    let months_between_halvings = months_to_milliseconds(months_between_halvings) as u64;

    println!("Wallet address: ");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .expect("stdin: Failed to read line");
    let wallet_address: String = buf.trim().parse().unwrap();
    let mut target = [0u8; 20];
    let source = wallet_address.as_bytes();

    for (target_elem, source_elem) in target.iter_mut().zip(source.iter().take(20)) {
        *target_elem = *source_elem;
    }
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

    println!("==============Blockchain successfully created===============");

    println!("{:#?} ", chain_config);
    println!("{:#?} ", node_config);
    let folder_path = "data";
    let file_name = String::from("chain_config.json");
    let file_name = file_name.as_str();
    std::fs::create_dir_all(folder_path).unwrap();
    let file_path = Path::new(folder_path).join(file_name);
    let mut file = File::create(file_path).unwrap();

    let content = serde_json::to_string(&chain_config).expect("Failed to serialize chain config");
    file.write_all(content.as_bytes()).unwrap();

    println!("Data has been written to the file.");
    println!("chain config saved to : {file_name}");
    (chain_config, node_config)
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

pub async fn run_node(port: &str) {
    let addr = String::from("127.0.0.1:") + port;
    println!("✔️ Running node in: {addr}");

    let node_service = NodeService::default();
    Server::builder()
        .add_service(NodeServer::new(node_service))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
