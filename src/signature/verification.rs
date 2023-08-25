use ethers::types::{Address, Signature};

pub fn verify_signature(msg: &str, signature: &Signature, from: &Address) -> bool {
    let recover_addr = signature.recover(msg).unwrap();
    recover_addr == *from
}
