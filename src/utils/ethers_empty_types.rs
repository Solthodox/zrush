use ethers::types::{Address, Signature, U256};

pub fn U256_ZERO() -> U256 {
    U256::from(0)
}

pub fn ADDRESS_ZERO() -> Address {
    Address::from([0u8; 20])
}

pub fn EMPTY_SIGNATURE() -> Signature {
    Signature {
        r: U256_ZERO(),
        s: U256_ZERO(),
        v: 0u64,
    }
}
