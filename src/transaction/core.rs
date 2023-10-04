use ethers::types::{Address, Signature, U256};
use serde_derive::{Deserialize, Serialize};

use crate::node::memory::NodeMemory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    from: Address,
    to: Address,
    amount: U256,
    fee_amount: U256,
    fee_receiver: Address,
    signature: Signature,
}

impl Transaction {
    pub fn new(
        from: Address,
        to: Address,
        amount: U256,
        fee_amount: U256,
        signature: Signature,
    ) -> Transaction {
        let fee_receiver = Address::from([0u8; 20]);
        Transaction {
            from,
            to,
            amount,
            fee_amount,
            fee_receiver,
            signature,
        }
    }

    pub fn genesis_tx(amount: U256, receiver: Address) -> Transaction {
        Transaction::new(
            Address::from([0u8; 20]),
            receiver,
            amount,
            U256::from(0),
            Signature {
                r: U256::from(0),
                s: U256::from(0),
                v: 0u64,
            },
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
}
