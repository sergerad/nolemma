use alloy_primitives::{keccak256, Address as AlloyAddress};
use secp256k1::PublicKey;
use serde::{Deserialize, Serialize};

/// A newtype wrapper around an Ethereum address.
/// Allows conversion from a public key.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Address(AlloyAddress);

impl From<PublicKey> for Address {
    fn from(pk: PublicKey) -> Self {
        let digest = keccak256(&pk.serialize_uncompressed()[1..]);
        Address(AlloyAddress::from_slice(&digest[12..]))
    }
}

impl Address {
    /// Generates a random address.
    pub fn random() -> Address {
        Address(AlloyAddress::random())
    }
}
