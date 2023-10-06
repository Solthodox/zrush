use ethers::types::{Address, Signature, U256};
use serde_derive::{Deserialize, Serialize};

use crate::{
    node::memory::NodeMemory,
    utils::{
        ethers_empty_types::{ADDRESS_ZERO, EMPTY_SIGNATURE, U256_ZERO},
        hashing::hash,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    from: Address,
    to: Address,
    amount: U256,
    fee_amount: U256,
    fee_receiver: Address,
    signature: Signature,
    timestamp: u64,
}

impl Transaction {
    pub fn new(
        from: Address,
        to: Address,
        amount: U256,
        fee_amount: U256,
        signature: Signature,
        timestamp: u64,
        fee_receiver: Address,
    ) -> Transaction {
        Transaction {
            from,
            to,
            amount,
            fee_amount,
            fee_receiver,
            signature,
            timestamp,
        }
    }

    pub fn genesis_tx(amount: U256, receiver: Address, timestamp: u64) -> Transaction {
        Transaction::new(
            ADDRESS_ZERO(),
            receiver,
            amount,
            U256_ZERO(),
            EMPTY_SIGNATURE(),
            timestamp,
            ADDRESS_ZERO(),
        )
    }

    pub fn from(&self) -> &Address {
        &self.from
    }

    pub fn to(&self) -> &Address {
        &self.to
    }

    pub fn amount(&self) -> &U256 {
        &self.amount
    }
    pub fn fee_amount(&self) -> &U256 {
        &self.fee_amount
    }

    pub fn fee_receiver(&self) -> &Address {
        &self.fee_receiver
    }
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn verify(&self, mem: &NodeMemory) -> bool {
        let from = self.from();
        let balance_from = mem.balance_of(from);
        let amount = self.amount();

        balance_from >= *amount
    }

    pub fn get_merkle(mempool: &Vec<Transaction>) -> String {
        let mut merkle = Vec::new();

        for t in mempool {
            let hash = hash(t);
            merkle.push(hash);
        }

        if merkle.len() % 2 == 1 {
            let last = merkle.last().cloned().unwrap();
            merkle.push(last);
        }

        while merkle.len() > 1 {
            let mut h1 = merkle.remove(0);
            let mut h2 = merkle.remove(0);
            h1.push_str(&mut h2);
            let nh = hash(&h1);
            merkle.push(nh);
        }
        merkle.pop().unwrap()
    }
}
