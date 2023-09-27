use ethers::types::{Address, U256};
use std::collections::HashMap;
use std::ops::Add;
use std::sync::Mutex;

use crate::block::block::Block;

#[derive(Debug)]
pub enum NodeMemoryError {
    CacheError(String),
}

#[derive(Debug, Default)]
pub struct NodeMemory {
    cache: Mutex<NodeCache>,
}

#[derive(Debug, Default)]
struct NodeCache {
    balances: HashMap<Address, U256>,
    nonces: HashMap<Address, U256>,
    block_difficulty: U256,
    block_height: U256,
    block_reward: U256,
}

impl NodeMemory {
    pub fn new() -> NodeMemory {
        let cache = NodeCache::default();
        NodeMemory {
            cache: Mutex::new(cache),
        }
    }

    pub fn balance_of(&self, addr: &Address) -> U256 {
        let binding = U256::from(0);
        match self
            .cache
            .lock()
            .unwrap()
            .balances
            .get(addr) {
                Some(n) => n.clone(),
                None => binding
            }
    }

    pub fn current_nonce(&self, addr: &Address) -> U256 {
        let binding = U256::from(0);
        match self.cache.lock().unwrap().nonces.get(addr){
            Some(n) => n.clone(),
            None => binding
        }
    }

    pub fn block_difficulty(&self) -> U256 {
        self.cache.lock().unwrap().block_difficulty.clone()
    }

    pub fn block_height(&self) -> U256 {
        self.cache.lock().unwrap().block_height.clone()
    }

    pub fn block_reward(&self) -> U256 {
        self.cache.lock().unwrap().block_reward.clone()
    }

    pub fn set_balance(&mut self, addr: &Address, amount: &U256) {
        let _ = &mut self.cache.lock().unwrap().balances.insert(*addr, *amount);
    }

    pub fn increment_nonce(&mut self, addr: &Address) {
        let binding = U256::from(0);
        let current_nonce = self.current_nonce(addr);
        let _ = &mut self
            .cache
            .lock()
            .unwrap()
            .nonces
            .insert(addr.clone(), current_nonce.add(1));
    }

    pub fn set_block_height(&mut self, height: &U256) {
        let mut block_height = self.cache.lock().unwrap().block_height;
        block_height = *height;
    }

    pub fn set_block_reward(&mut self, reward: &U256) {
        let mut block_reward = self.cache.lock().unwrap().block_reward;
        block_reward = *reward;
    }

    pub fn set_block_difficulty(&mut self, difficulty: &U256) {
        let mut block_difficulty = self.cache.lock().unwrap().block_difficulty;
        block_difficulty = *difficulty;
    }

    pub fn increment_block_height(&mut self) {
        let mut block_height = self.cache.lock().unwrap().block_height;
        block_height = block_height.add(1);
    }

    pub fn cache(chain: &str) -> Result<NodeMemory, NodeMemoryError> {
        println!("Caching memory...");
        let mut node_memory = NodeMemory::new();
        let chain = serde_json::from_str::<Vec<Block>>(&chain)
            .map_err(|_| NodeMemoryError::CacheError(String::from("Could not read chain data")))?;
        let zero_address = Address::from([0u8; 20]);
        let len = chain.len();
        for (i, block) in chain.iter().enumerate() {
            println!("{:#?}", block);
            if i == len - 1 {
                node_memory.set_block_height(block.height());
                node_memory.set_block_difficulty(block.difficulty());
                node_memory.set_block_reward(block.reward());
            }
            let transactions = block.transactions();
            for tx in transactions.iter() {
                let amount = tx.amount();
                let from = tx.from();
                if *from != *&zero_address {
                    let sender_balance = node_memory.balance_of(from);
                    let new_balance = sender_balance.checked_sub(*amount).unwrap();
                    node_memory.set_balance(from, &new_balance);
                    node_memory.increment_nonce(from);
                }
                let to = tx.to();
                let receiver_balance = node_memory.balance_of(to);
                let new_balance = receiver_balance.checked_add(*amount).unwrap();
                node_memory.set_balance(to, &new_balance);

                let fee_amount = tx.fee_amount();
                let fee_receiver = tx.fee_receiver();
                if *fee_receiver != *&zero_address {
                    let sender_balance = node_memory.balance_of(from);
                    let new_sender_balance = sender_balance.checked_sub(*fee_amount).unwrap();
                    node_memory.set_balance(from, &new_sender_balance);

                    let receiver_balance = node_memory.balance_of(fee_receiver);
                    let new_receiver_balance =
                        receiver_balance.checked_add(receiver_balance).unwrap();
                    node_memory.set_balance(to, &new_receiver_balance);
                }
            }
        }
        Ok(node_memory)
    }
}
