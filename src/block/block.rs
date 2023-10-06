use crate::{node::memory::NodeMemory, transaction::core::Transaction};
pub const ZERO_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";
use chrono::Utc;
use ethers::types::{Address, U256};
use serde_derive::{Deserialize, Serialize};

// TODO: order functions
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BlockHeader {
    pub hash: String,
    pub timestamp: u64,
    pub nonce: U256,
    pub pre_hash: String,
    pub merkle: String,
    pub difficulty: U256,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Block {
    header: BlockHeader,
    height: U256,
    reward: U256,
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
        reward: U256,
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
            reward,
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
            reward: block_reward,
            transactions: vec![Transaction::genesis_tx(block_reward, receiver)],
        }
    }

    pub fn validate(&self, mem: &NodeMemory) -> bool {
        let difficulty_ok = *self.difficulty() == mem.block_difficulty();
        let height_ok = *self.difficulty() == mem.block_height();
        let reward_ok = *self.reward() == mem.block_reward();
        true
    }

    pub fn merkle_tx(txs: &Vec<Transaction>) -> String {
        todo!()
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

    pub fn hash(&self) -> &String {
        &self.header().hash
    }

    pub fn merkle(&self) -> &String {
        &self.header().merkle
    }

    pub fn pre_hash(&self) -> &String {
        &self.header().pre_hash
    }

    pub fn nonce(&self) -> &U256 {
        &self.header().nonce
    }

    pub fn difficulty(&self) -> &U256 {
        &self.header().difficulty
    }

    pub fn reward(&self) -> &U256 {
        &self.reward
    }

    pub fn mine(&mut self, difficulty: U256) {}
}
