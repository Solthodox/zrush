use std::fs;
use std::io::stdin;
use std::path::Path;

use ethers::signers::{Signer, Wallet, WalletError};
use ethers::types::Address;
use rand::thread_rng;

pub fn create_wallet() -> Result<Address, WalletError> {
    println!("Creating wallet...");

    println!("Wallet password: ");
    let dir = Path::new(".keys");
    let mut buf = String::new();
    let _ = stdin()
        .read_line(&mut buf)
        .map_err(|_| "stdin: Failed to read password");

    let wallet_password = buf.as_bytes();
    println!("Do not forget your password!");
    match fs::create_dir(".keys") {
        Ok(_) => (),
        Err(_) => (),
    }
    let res = Wallet::new_keystore(&dir, &mut thread_rng(), &wallet_password, None);
    match res {
        Ok((wallet, _)) => {
            let addr = wallet.address();
            return Ok(addr);
        }
        Err(e) => return Err(e),
    }
}
