use ethers::types::{Address, Signature, U256};

pub const U256_ZERO: U256 = U256::from(0);

pub const ADDRESS_ZERO: Address = Address::from([0u8; 20]);

pub const EMPTY_SIGNATURE: Signature = Signature {
    r: U256::from(0),
    s: U256::from(0),
    v: 0u64,
};
