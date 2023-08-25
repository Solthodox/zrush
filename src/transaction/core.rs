use ethers::types::{Address, Signature, U256};
use serde_derive::Serialize;

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
