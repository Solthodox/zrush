use crate::{
    node::memory::NodeMemory,
    transaction::core::Transaction,
    utils::{
        files::{read_from_file, write_to_file},
        hashing::hash,
    },
};
pub const ZERO_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";
use chrono::Utc;
use ethers::types::{Address, U256};
use serde_derive::{Deserialize, Serialize};

// TODO: order functions
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BlockHeader {
    pub hash: String,
    pub timestamp: u64,
    pub nonce: U256,
    pub pre_hash: String,
    pub merkle: String,
    pub difficulty: U256,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
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
    pub fn genesis_block(block_reward: U256, receiver: Address, timestamp: u64) -> Block {
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
            transactions: vec![Transaction::genesis_tx(block_reward, receiver, timestamp)],
        }
    }

    pub fn validate(&self, mem: &mut NodeMemory) -> bool {
        if !(*self.difficulty() == mem.block_difficulty()) {
            return false;
        }
        if !(*self.difficulty() == mem.block_height()) {
            return false;
        }
        if !(*self.reward() == mem.block_reward()) {
            return false;
        }

        mem.increment_block_height();
        let blocks = read_from_file("data/storage", "chain_data.json").unwrap();
        let mut blocks: Vec<Block> = serde_json::from_str(&blocks).unwrap();
        let last_block = blocks.last().unwrap();
        if self.pre_hash() != last_block.hash() {
            return false;
        }
        if self.timestamp() != last_block.timestamp() {
            return false;
        }
        let transactions = self.transactions();
        for tx in transactions.iter() {
            match tx.verify(mem) {
                true => {}
                false => return false,
            }
        }
        let _merkle = Transaction::get_merkle(&transactions);
        if *self.merkle() != _merkle {
            return false;
        }
        let expected_hash = format!("{}{}", _merkle, self.nonce());
        if *self.hash() != hash(&expected_hash) {
            return false;
        }
        blocks.push(self.clone());
        let content = serde_json::to_string(&blocks).unwrap();
        write_to_file("data/storage", "chain_data.json", &content).unwrap();
        true
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

    pub fn timestamp(&self) -> &u64 {
        &self.header().timestamp
    }

    pub fn mine(&mut self, difficulty: U256) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::memory::NodeMemory;
    use crate::transaction::core::Transaction;

    #[test]
    fn test_new_block() {
        // Create a new block and verify that its properties are set correctly.
        let block = Block::new(
            "block_hash".to_string(),
            1234567890,
            U256::from(42),
            "previous_hash".to_string(),
            "merkle_root".to_string(),
            U256::from(100),
            U256::from(1),
            U256::from(10),
            vec![Transaction::default()],
        );

        assert_eq!(block.hash(), "block_hash");
        assert_eq!(block.timestamp(), &1234567890);
        assert_eq!(block.nonce(), &U256::from(42));
        assert_eq!(block.pre_hash(), "previous_hash");
        assert_eq!(block.merkle(), "merkle_root");
        assert_eq!(block.difficulty(), &U256::from(100));
        assert_eq!(block.height(), &U256::from(1));
        assert_eq!(block.reward(), &U256::from(10));
        assert_eq!(block.transactions().len(), 1);
    }

    #[test]
    fn test_genesis_block() {
        // Create a genesis block and verify its properties.
        let receiver = Address::random();
        let timestamp = 1234567890;
        let block_reward = U256::from(10);

        let genesis_block = Block::genesis_block(block_reward, receiver, timestamp);

        assert_eq!(genesis_block.hash(), ZERO_HEX);
        assert_eq!(genesis_block.timestamp(), &timestamp);
        assert_eq!(genesis_block.nonce(), &U256::from(0));
        assert_eq!(genesis_block.pre_hash(), ZERO_HEX);
        assert_eq!(genesis_block.merkle(), ZERO_HEX);
        assert_eq!(genesis_block.difficulty(), &U256::from(1));
        assert_eq!(genesis_block.height(), &U256::from(0));
        assert_eq!(genesis_block.reward(), &block_reward);
        assert_eq!(genesis_block.transactions().len(), 1);
    }

    // Write more tests to cover other methods like validate, mine, etc.
}