use crate::{
    block::core::Block,
    node::memory::NodeMemory,
    signature::verification::verify_signature,
    transaction::core::Transaction,
    utils::{
        banner::print_banner,
        ethers_empty_types::ADDRESS_ZERO,
        files::{read_from_file, write_to_file},
        timestamp::{current_timestamp, months_to_milliseconds},
    },
    wallet::core::create_wallet,
    p2p::core::{connect_node, propagate_transaction, propagate_block}
};
use ethers::types::{Address, Signature, U256};
use serde_derive::Serialize;
use std::{io::stdin,sync::Mutex, thread};
use tokio::runtime;
use tonic::{transport::Server, Code, Request, Response, Status};
use super::node_proto::node_proto;


use node_proto::{
    node_client::NodeClient,
    node_server::{Node, NodeServer},
    AddBlockRequest, BlockResponse, NodeInfoRequest, RequestNodeInfoResponse, RequestSyncResponse,
    SyncRequest, TransactionRequest, TransactionResponse,
};


#[derive(Debug)]
pub enum NodeError {
    SyncError(String),
    NetworkError(String),
    InvalidConfigInput(String),
}

#[derive(Default)]
pub struct NodeService {
    memory: Mutex<NodeMemory>,
}

#[tonic::async_trait]
impl Node for NodeService {
    async fn request_send_transaction(
        &self,
        req: Request<TransactionRequest>,
    ) -> Result<Response<TransactionResponse>, Status> {
        let client_address = req.remote_addr();
        let tx = req.into_inner();
        let tx_copy = tx.clone();

        // Parse the gRPC transaction request
        let parsed_tx = match parse_grpc_transaction_request(tx) {
            Ok(parsed_tx) => parsed_tx,
            Err(err) => return Err(Status::new(Code::InvalidArgument, err)),
        };

        println!("{:#?}", parsed_tx);

        let from = parsed_tx.from();
        let to = parsed_tx.to();
        let amount = parsed_tx.amount();
        let mut mem = self.memory.lock().unwrap();

        // Construct the message for signature verification
        let nonce = mem.current_nonce(from);
        let msg = format!("zrush_signed_message:{}{}{:?}{:?}", from, to, amount, nonce);

        // Check transaction validity and signature
        if !parsed_tx.verify(&mem) {
            return Err(Status::new(
                Code::InvalidArgument,
                "Invalid transaction amount",
            ));
        } else if !verify_signature(&msg, parsed_tx.signature(), from) {
            return Err(Status::new(Code::InvalidArgument, "Invalid signature"));
        }

        // Update sender's balance and nonce
        let sender_balance = mem.balance_of(from);
        let new_balance = sender_balance.checked_sub(*amount).unwrap();
        mem.set_balance(from, &new_balance);
        mem.increment_nonce(from);

        // Update receiver's balance
        let receiver_balance = mem.balance_of(to);
        let new_balance = receiver_balance.checked_add(*amount).unwrap();
        mem.set_balance(to, &new_balance);

        // Push the transaction to the mempool
        mem.push_to_mempool(&parsed_tx);

        // Use `await` here to wait for `connect_node` to complete
        let handle = thread::spawn(move || {
            // Create a tokio runtime
            let rt = runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            // Run the async function within the tokio runtime
            rt.block_on(async {
                connect_node(client_address).await.unwrap();
                propagate_transaction(tx_copy).await.unwrap();
            });
        });

        handle.join().unwrap();

        // Return a successful response
        Ok(Response::new(TransactionResponse {}))
    }
    async fn request_add_block(
        &self,
        req: Request<AddBlockRequest>,
    ) -> Result<Response<BlockResponse>, Status> {
        let client_address = req.remote_addr();
        let block = req.into_inner();
        let block_copy = block.clone();
        let block = parse_grpc_block_request(block)
            .map_err(|err| Status::new(Code::InvalidArgument, err))?;
        let mem = &mut self.memory.lock().unwrap();

        // Use `await` here to wait for `connect_node` to complete
        let handle = thread::spawn(move || {
            // Create a tokio runtime
            let rt = runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            // Run the async function within the tokio runtime
            rt.block_on(async {
                connect_node(client_address).await.unwrap();
                propagate_block(block_copy).await.unwrap();
            });
        });

        handle.join().unwrap();
        match block.validate(&mem) {
            true => {
                let chain = read_from_file("data/storage", "chain_data.json").unwrap();
                let mut chain: Vec<Block> = serde_json::from_str(&chain).unwrap();
                chain.push(block);
                let content = serde_json::to_string(&chain).unwrap();
                write_to_file("data/", "chain_data.json", &content).unwrap();
                return Ok(Response::new(BlockResponse {}));
            }
            false => {
                return Err(Status::new(
                    Code::Unauthenticated,
                    "Couldn't  validate block",
                ))
            }
        }
    }

    async fn request_sync(
        &self,
        _req: Request<SyncRequest>,
    ) -> Result<Response<RequestSyncResponse>, Status> {
        let client_address = _req.remote_addr();
        // Use `await` here to wait for `connect_node` to complete
        let handle = thread::spawn(move || {
            // Create a tokio runtime
            let rt = runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            // Run the async function within the tokio runtime
            rt.block_on(async {
                connect_node(client_address).await.unwrap();
            });
        });

        handle.join().unwrap();
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

    async fn request_node_info(
        &self,
        _req: Request<NodeInfoRequest>,
    ) -> Result<Response<RequestNodeInfoResponse>, Status> {
        Ok(Response::new(RequestNodeInfoResponse {
            address: self.memory.lock().unwrap().node_address(),
        }))
    }
}

fn parse_grpc_block_request(block: AddBlockRequest) -> Result<Block, &'static str> {
    let hash: String = block.hash.try_into().map_err(|_| "Invalid hash")?;
    let timestamp: u64 = block
        .timestamp
        .try_into()
        .map_err(|_| "Invalid timestamp")?;
    let nonce: [u8; 32] = block.nonce.try_into().map_err(|_| "Invalid nonce")?;
    let pre_hash: String = block.pre_hash.try_into().map_err(|_| "Invalid pre hash")?;
    let merkle: String = block.merkle.try_into().map_err(|_| "Invalid merkle")?;
    let difficulty: [u8; 32] = block
        .difficulty
        .try_into()
        .map_err(|_| "Invalid difficulty")?;
    let height: [u8; 32] = block.height.try_into().map_err(|_| "Invalid height")?;
    let reward: [u8; 32] = block.reward.try_into().map_err(|_| "Invalid reward")?;

    // Use a concise map to parse transactions
    let parsed_transactions: Result<Vec<Transaction>, &'static str> = block
        .transactions
        .iter()
        .map(|tx| parse_grpc_transaction_request(tx.clone()).map_err(|err| err))
        .collect();

    Ok(Block::new(
        hash,
        timestamp,
        U256::from(nonce),
        pre_hash,
        merkle,
        U256::from(difficulty),
        U256::from(height),
        U256::from(reward),
        parsed_transactions?,
    ))
}

fn parse_grpc_transaction_request(tx: TransactionRequest) -> Result<Transaction, &'static str> {
    let from: [u8; 20] = tx.from.try_into().map_err(|_| "Invalid from")?;
    let to: [u8; 20] = tx.to.try_into().map_err(|_| "Invalid to")?;
    let amount: [u8; 32] = tx.amount.try_into().map_err(|_| "Invalid amount")?;
    let fee: [u8; 32] = tx.fee.try_into().map_err(|_| "Invalid fee")?;
    let signature = tx.signature.unwrap();

    let r: [u8; 32] = signature.r.try_into().map_err(|_| "Invalid r")?;
    let s: [u8; 32] = signature.s.try_into().map_err(|_| "Invalid s")?;
    let r = U256::from(r);
    let s = U256::from(s);
    let v = signature.v;

    Ok(Transaction::new(
        Address::from(from),
        Address::from(to),
        U256::from(amount),
        U256::from(fee),
        Signature { r, s, v },
        current_timestamp(),
        ADDRESS_ZERO(),
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

pub fn create_new_blockchain() -> Result<(), NodeError> {
    print_banner();
    println!("Welcome to zrush v1.1\n");

    let (chain_config, wallet_address) = config_blockchain()
        .map_err(|err_msg| NodeError::InvalidConfigInput(err_msg.to_string()))?;

    let first_block = Block::genesis_block(
        chain_config.initial_block_reward,
        wallet_address,
        current_timestamp(),
    );
    let chain = vec![first_block];

    let serialization = serde_json::to_string(&chain).map_err(|_| {
        NodeError::InvalidConfigInput("Failed to serialize genesis block".to_string())
    })?;

    let _ = write_to_file("./data/storage", "chain_data.json", &serialization);

    Ok(())
}

fn config_blockchain() -> Result<(ChainConfig, Address), &'static str> {
    println!("Name:");
    let name = read_input("Invalid name")?.trim().to_lowercase();

    println!("Chain id:");
    let chain_id = read_input("Invalid chain id")?
        .trim()
        .parse::<u8>()
        .map_err(|_| "Invalid chain id")?;

    println!("Seconds between blocks:");
    let seconds_between_blocks = read_input("Invalid seconds between blocks")?
        .trim()
        .parse::<u8>()
        .map_err(|_| "Invalid seconds between blocks")?;

    println!("Initial block reward:");
    let initial_block_reward = read_input("Invalid initial block reward")?;
    let initial_block_reward = parse_u256(&initial_block_reward)?;

    println!("Months between halvings:");
    let months_between_halvings = read_input("Invalid months between halving")?
        .trim()
        .parse::<i64>()
        .map_err(|_| "Invalid months between halving")?;
    let months_between_halvings = months_to_milliseconds(months_between_halvings) as u64;

    let addr = create_wallet().unwrap();
    let creation_timestamp = current_timestamp();

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
    println!("Chain config saved to: data/chain_config.json");

    Ok((chain_config, addr))
}

fn read_input(error_message: &str) -> Result<String, &'static str> {
    let mut buf = String::new();
    stdin()
        .read_line(&mut buf)
        .map_err(|_| "stdin: Failed to read input")?;
    Ok(buf)
}

fn parse_u256(input: &str) -> Result<U256, &'static str> {
    let mut target = [0u8; 32];
    let source = input.as_bytes();
    let bytes_to_copy = source.len().min(32);

    target[..bytes_to_copy].copy_from_slice(&source[..bytes_to_copy]);
    Ok(U256::from(&target))
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

pub async fn run_node(port: String, memory: Mutex<NodeMemory>) -> Result<(), NodeError> {
    let addr = String::from("127.0.0.1:") + port.as_str();
    memory.lock().unwrap().set_node_address(addr.clone());
    println!("✔️ Running node in: {addr}");
    let parsed_addr = addr
        .parse()
        .map_err(|_| NodeError::InvalidConfigInput(String::from("Invalid port")))?;

    let node_service = NodeService { memory };
    Server::builder()
        .add_service(NodeServer::new(node_service))
        .serve(parsed_addr)
        .await
        .map_err(|err| NodeError::NetworkError(err.to_string()))?;
    Ok(())
}
