use ethers::types::{Address, U256};
use std::collections::HashMap;
use std::ops::Add;
use std::sync::Mutex;

use crate::block::core::Block;
use crate::transaction::core::Transaction;
use crate::utils::ethers_empty_types::ADDRESS_ZERO;

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
        let block_height = self.cache.lock().unwrap().block_height;
        let _ = block_height.add(1);
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

        let mut node_memory = NodeMemory::new();
        let len = chain.len();

        for (i, block) in chain.iter().enumerate() {
            println!("{:#?}", block);
            if i == len - 1 {
                node_memory.update_last_block_info(block);
            }

            node_memory.process_transactions(block.transactions());
        }

        Ok(node_memory)
    }

    pub fn update(&mut self, transactions: &Vec<Transaction>) {
        for tx in transactions.iter() {
            
        }
    }

    fn update_last_block_info(&mut self, block: &Block) {
        self.set_block_height(block.height());
        self.set_block_difficulty(block.difficulty());
        self.set_block_reward(block.reward());
    }

    fn process_transactions(&mut self, transactions: &[Transaction]) {
        for tx in transactions.iter() {
            self.process_sender(tx);
            self.process_receiver(tx);
            self.process_fee(tx);
        }
    }

    fn process_sender(&mut self, tx: &Transaction) {
        let amount = tx.amount();
        let from = tx.from();
        if *from != ADDRESS_ZERO() {
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

    fn process_fee(&mut self,tx: &Transaction) {
        let fee_amount = tx.fee_amount();
        let fee_receiver = tx.fee_receiver();
        if *fee_receiver != ADDRESS_ZERO() {
            let sender_balance = self.balance_of(fee_receiver);
            let new_sender_balance = sender_balance.checked_sub(*fee_amount).unwrap();
            self.set_balance(fee_receiver, &new_sender_balance);

            let receiver_balance = self.balance_of(fee_receiver);
            let new_receiver_balance = receiver_balance.checked_add(*fee_amount).unwrap();
            self.set_balance(fee_receiver, &new_receiver_balance);
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::utils::ethers_empty_types::{U256_ZERO, EMPTY_SIGNATURE};

    use super::*;
    use ethers::types::{Address, H160, U256};

    #[test]
    fn test_new_node_memory() {
        let node_memory = NodeMemory::new();
        let cache = node_memory.cache.lock().unwrap();
        let mempool = node_memory.mempool.lock().unwrap();
        let node_address = node_memory.node_address.lock().unwrap();

        assert!(cache.balances.is_empty());
        assert!(cache.nonces.is_empty());
        assert_eq!(cache.block_difficulty, U256::zero());
        assert_eq!(cache.block_height, U256::zero());
        assert_eq!(cache.block_reward, U256::zero());

        assert!(mempool.is_empty());

        assert!(node_address.is_empty());
    }

    #[test]
    fn test_balance_of() {
        let node_memory = NodeMemory::new();
        let address = H160::zero();
        let balance = node_memory.balance_of(&address);
        assert_eq!(balance, U256::zero());
    }

    #[test]
    fn test_current_nonce() {
        let node_memory = NodeMemory::new();
        let address = H160::zero();
        let nonce = node_memory.current_nonce(&address);
        assert_eq!(nonce, U256::zero());
    }

    #[test]
    fn test_block_difficulty() {
        let node_memory = NodeMemory::new();
        let difficulty = node_memory.block_difficulty();
        assert_eq!(difficulty, U256::zero());
    }

    #[test]
    fn test_block_height() {
        let node_memory = NodeMemory::new();
        let height = node_memory.block_height();
        assert_eq!(height, U256::zero());
    }

    #[test]
    fn test_block_reward() {
        let node_memory = NodeMemory::new();
        let reward = node_memory.block_reward();
        assert_eq!(reward, U256::zero());
    }

    #[test]
    fn test_set_balance() {
        let mut node_memory = NodeMemory::new();
        let address = H160::from_low_u64_be(123);
        let amount = U256::from(100);

        node_memory.set_balance(&address, &amount);
        let balance = node_memory.balance_of(&address);
        assert_eq!(balance, amount);
    }

    #[test]
    fn test_push_to_mempool() {
        let mut node_memory = NodeMemory::new();
        let transaction = Transaction::new(
            ADDRESS_ZERO(),
            ADDRESS_ZERO(),
            U256_ZERO(),
            U256_ZERO(),
            EMPTY_SIGNATURE(),
            0u64,
            ADDRESS_ZERO(),
        ); // You need to create a transaction instance.
        node_memory.push_to_mempool(&transaction);

        let mempool = node_memory.mempool.lock().unwrap();
        assert_eq!(mempool.len(), 1);
    }

    #[test]
    fn test_increment_nonce() {
        let mut node_memory = NodeMemory::new();
        let address = H160::zero();

        node_memory.increment_nonce(&address);
        let nonce = node_memory.current_nonce(&address);
        assert_eq!(nonce, U256::one());
    }

    // Similar tests for set_block_height, set_block_reward, and set_block_difficulty methods.

    #[test]
    fn test_node_address() {
        let node_memory = NodeMemory::new();
        let address = node_memory.node_address();
        assert_eq!(address, "");
    }

    #[test]
    fn test_set_node_address() {
        let mut node_memory = NodeMemory::new();
        let address = "0x1234567890abcdef".to_string();

        node_memory.set_node_address(address.clone());
        let retrieved_address = node_memory.node_address();
        assert_eq!(retrieved_address, address);
    }

    // Test the cache method when deserialization is successful.
    #[test]
    fn test_cache_successful() {
        let chain_json = "[{\"height\": 1, \"difficulty\": \"0x10\", \"reward\": \"0x20\", \"transactions\": []}]";
        let result = NodeMemory::cache(chain_json);
        assert!(result.is_ok());

        let node_memory = result.unwrap();
        assert_eq!(node_memory.block_height(), U256::from(1));
        assert_eq!(node_memory.block_difficulty(), U256::from(16));
        assert_eq!(node_memory.block_reward(), U256::from(32));
    }

    // Test the cache method when deserialization fails.
    #[test]
    fn test_cache_deserialization_error() {
        let invalid_chain_json = "invalid_json";
        let result = NodeMemory::cache(invalid_chain_json);
        assert!(result.is_err());
       /*  assert_eq!(
            result.err().unwrap(),
            NodeMemoryError::CacheError(String::from("Could not read chain data"))
        ); */
    }

    // Write more tests for the update and private helper methods as needed.
}