use ethers::types::{Address, Signature};

pub fn verify_signature(msg: &str, signature: &Signature, from: &Address) -> bool {
    let recover_addr = signature.recover(msg).unwrap();
    recover_addr == *from
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::utils::signer::{Signer, LocalWallet};
    use ethers::core::types::Transaction;

    #[test]
    fn test_successful_signature_verification() {
        // Create a message, sign it, and then verify the signature
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let message = "Hello, world!";
        let signature = wallet.sign_message(message).unwrap();
        let from = wallet.address();

        let verified = verify_signature(message, &signature, &from);
        assert!(verified);
    }

    #[test]
    fn test_failed_signature_verification() {
        // Create two different wallets and sign the same message with both.
        // Then, try to verify the message with the wrong wallet's address.
        let wallet1 = LocalWallet::new(&mut rand::thread_rng());
        let wallet2 = LocalWallet::new(&mut rand::thread_rng());
        let message = "Hello, world!";
        let signature1 = wallet1.sign_message(message).unwrap();
        let from2 = wallet2.address();

        let verified = verify_signature(message, &signature1, &from2);
        assert!(!verified);
    }

    #[test]
    fn test_invalid_signature() {
        // Try to verify an invalid signature
        let message = "Hello, world!";
        let invalid_signature = Signature::empty();
        let from = Address::zero();

        let verified = verify_signature(message, &invalid_signature, &from);
        assert!(!verified);
    }

    // Add more test cases to cover other scenarios as needed.
}