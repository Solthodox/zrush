use ethers::types::{Address, U256};
use std::collections::HashMap;
use std::ops::Add;
use std::sync::Mutex;

use crate::block::block::Block;
use crate::transaction::core::Transaction;

#[derive(Debug)]
pub enum NodeMemoryError {
    CacheError(String),
}

#[derive(Debug, Default)]
pub struct NodeMemory {
    cache: Mutex<NodeCache>,
    mempool: Mutex<Vec<Transaction>>,
    node_address: Mutex<String>,
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
            mempool: Mutex::new(vec![]),
            node_address: Mutex::new(String::new()),
        }
    }

    pub fn balance_of(&self, addr: &Address) -> U256 {
        let binding = U256::from(0);
        match self.cache.lock().unwrap().balances.get(addr) {
            Some(n) => n.clone(),
            None => binding,
        }
    }

    pub fn current_nonce(&self, addr: &Address) -> U256 {
        let binding = U256::from(0);
        match self.cache.lock().unwrap().nonces.get(addr) {
            Some(n) => n.clone(),
            None => binding,
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

    pub fn push_to_mempool(&mut self, tx: &Transaction) {
        let _ = &mut self.mempool.lock().unwrap().push(tx.clone());
    }

    pub fn increment_nonce(&mut self, addr: &Address) {
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

    pub fn node_address(&self) -> String {
        self.node_address.lock().unwrap().clone()
    }

    pub fn set_node_address(&mut self, addr: String) {
        let mut node_address = self.node_address.lock().unwrap();
        *node_address = addr;
    }

    pub fn cache(chain: &str) -> Result<NodeMemory, NodeMemoryError> {
        println!("Caching memory...");
        let chain = serde_json::from_str::<Vec<Block>>(&chain)
            .map_err(|_| NodeMemoryError::CacheError(String::from("Could not read chain data")))?;

        let zero_address = Address::from([0u8; 20]);
        let mut node_memory = NodeMemory::new();
        let len = chain.len();

        for (i, block) in chain.iter().enumerate() {
            println!("{:#?}", block);
            if i == len - 1 {
                node_memory.update_last_block_info(block);
            }

            node_memory.process_transactions(zero_address, block.transactions());
        }

        Ok(node_memory)
    }

    fn update_last_block_info(&mut self, block: &Block) {
        self.set_block_height(block.height());
        self.set_block_difficulty(block.difficulty());
        self.set_block_reward(block.reward());
    }

    fn process_transactions(&mut self, zero_address: Address, transactions: &[Transaction]) {
        for tx in transactions.iter() {
            self.process_sender(&zero_address, tx);
            self.process_receiver(tx);
            self.process_fee(&zero_address, tx);
        }
    }

    fn process_sender(&mut self, zero_address: &Address, tx: &Transaction) {
        let amount = tx.amount();
        let from = tx.from();
        if *from != *zero_address {
            let sender_balance = self.balance_of(from);
            let new_balance = sender_balance.checked_sub(*amount).unwrap();
            self.set_balance(from, &new_balance);
            self.increment_nonce(from);
        }
    }

    fn process_receiver(&mut self, tx: &Transaction) {
        let amount = tx.amount();
        let to = tx.to();
        let receiver_balance = self.balance_of(to);
        let new_balance = receiver_balance.checked_add(*amount).unwrap();
        self.set_balance(to, &new_balance);
    }

    fn process_fee(&mut self, zero_address: &Address, tx: &Transaction) {
        let fee_amount = tx.fee_amount();
        let fee_receiver = tx.fee_receiver();
        if *fee_receiver != *zero_address {
            let sender_balance = self.balance_of(fee_receiver);
            let new_sender_balance = sender_balance.checked_sub(*fee_amount).unwrap();
            self.set_balance(fee_receiver, &new_sender_balance);

            let receiver_balance = self.balance_of(fee_receiver);
            let new_receiver_balance = receiver_balance.checked_add(*fee_amount).unwrap();
            self.set_balance(fee_receiver, &new_receiver_balance);
        }
    }
}
