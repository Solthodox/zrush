use crate::transaction::core::Transaction;
pub const ZERO_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";
use chrono::Utc;
use ethers::types::{Address, U256};
use serde_derive::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct BlockHeader {
    hash: String,
    timestamp: u64,
    nonce: U256,
    pre_hash: String,
    merkle: String,
    difficulty: U256,
}

#[derive(Debug, Default, Serialize)]
pub struct Block {
    header: BlockHeader,
    height: U256,
    transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(
        hash: String,
        timestamp: u64,
        nonce: U256,
        pre_hash: String,
        merkle: String,
        difficulty: U256,
        height: U256,
        transactions: Vec<Transaction>,
    ) -> Block {
        Block {
            header: BlockHeader {
                hash,
                timestamp,
                nonce,
                pre_hash,
                merkle,
                difficulty,
            },
            height,
            transactions,
        }
    }
    pub fn genesis_block(block_reward: U256, receiver: Address) -> Block {
        let creation_timestamp = Utc::now().timestamp_millis() as u64;
        Block {
            header: BlockHeader {
                hash: String::from(ZERO_HEX),
                timestamp: creation_timestamp,
                nonce: U256::from(0),
                pre_hash: String::from(ZERO_HEX),
                merkle: String::from(ZERO_HEX),
                difficulty: U256::from(1),
            },
            height: U256::from(0),
            transactions: vec![Transaction::genesis_tx(block_reward, receiver)],
        }
    }

    pub fn header(&self) -> &BlockHeader {
        &self.header
    }

    pub fn height(&self) -> &U256 {
        &self.height
    }

    pub fn transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn mine(&mut self, difficulty: U256) {}
}
