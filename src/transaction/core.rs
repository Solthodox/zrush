use ethers::types::{Address, Signature, U256, H160};
use serde_derive::Serialize;

use crate::signature;

#[derive(Debug, Clone, Serialize)]
pub struct Transaction {
    from: Address,
    to: Address,
    amount: U256,
    fee: U256,
    signature: Signature,
}

impl Transaction {
    pub fn new(
        from: Address,
        to: Address,
        amount: U256,
        fee: U256,
        signature: Signature,
    ) -> Transaction {
        Transaction {
            from,
            to,
            amount,
            fee,
            signature,
        }
    }

    pub fn genesis_tx(amount: U256, receiver: Address) -> Transaction {
        Transaction::new(
            Address::from([0u8; 20]),
            receiver,
            amount,
            U256::from(0),
            Signature { r: U256::from(0), s: U256::from(0), v: 0u64 }
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
    pub fn fee(&self) -> &U256 {
        &self.fee
    }
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

}
