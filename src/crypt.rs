use ed25519_dalek::{self as dalek, Signer, Verifier, VerifyingKey};
use dalek::{Signature, SigningKey};
use std::str::FromStr;

/*
Cryptography module for NETCOM

A replaceable module meant for exposing an API to
some basic cryptography components.
Mainly signing, verifying and public key transformations
*/

/// Sign a message with your private key
pub fn sign(key: &[u8; 32], message: &str) -> String {
    let privkey = SigningKey::from_bytes(key);
    let signature = privkey.sign(message.as_bytes());
    signature.to_string()
}

/// Make sure that a signed messages matches 
/// with an expected message using some public key
pub fn verify(key: &[u8; 32], signed_message: &str, expected_message: &str) -> bool {
    if signed_message == "" {return false} // for when the client starts tweaking
    let ac_key = VerifyingKey::from_bytes(key).expect("Not a valid key");
    let sign = Signature::from_str(signed_message)
        .expect("Signed message wasn't valid");
    ac_key.verify(expected_message.as_bytes(), &sign).is_ok()
}

/// Converts a private key to its public variant
pub fn get_public(privkey: &[u8; 32]) -> [u8; 32] {
    let key = SigningKey::from_bytes(privkey);
    return key.verifying_key().as_bytes().clone();
}