use alloy_primitives::bytes::BufMut;
use alloy_primitives::U256;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId, Signature as SecpSignature};
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Message, Secp256k1};
use secp256k1::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::Address;

/// A recoverable seckp256k1 signature.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Signature {
    /// The r component of the signature.
    pub r: U256,
    /// The s component of the signature.
    pub s: U256,
    /// The recovery id of the signature.
    pub recovery_id: i32,
}

/// Converts a [Signature] into a [SecpSignature].
impl From<&Signature> for SecpSignature {
    fn from(signature: &Signature) -> Self {
        let mut buf = Vec::new();
        buf.put_slice(&signature.r.to_be_bytes::<32>());
        buf.put_slice(&signature.s.to_be_bytes::<32>());
        SecpSignature::from_compact(&buf).unwrap()
    }
}

/// Converts a [Signature] into a [RecoverableSignature].
impl From<&Signature> for RecoverableSignature {
    fn from(signature: &Signature) -> Self {
        let mut buf = Vec::new();
        buf.put_slice(&signature.r.to_be_bytes::<32>());
        buf.put_slice(&signature.s.to_be_bytes::<32>());
        RecoverableSignature::from_compact(
            &buf,
            RecoveryId::from_i32(signature.recovery_id).unwrap(),
        )
        .unwrap()
    }
}

/// An entity capable of signing messages using secp2561k.
pub struct Signer {
    pub sk: SecretKey,
    pub pk: PublicKey,
    pub address: Address,
}

/// Converts a string into a [Signer].
impl From<&str> for Signer {
    fn from(s: &str) -> Self {
        let sk = SecretKey::from_str(s).unwrap();
        let pk = PublicKey::from_secret_key_global(&sk);
        let address = Address::from(pk);
        Signer { sk, pk, address }
    }
}

impl Signer {
    /// Generates a random [Signer].
    pub fn random() -> Signer {
        let secp = Secp256k1::new();
        let (sk, pk) = secp.generate_keypair(&mut OsRng);
        let address = Address::from(pk);
        Signer { sk, pk, address }
    }

    /// Signs a digest using the [Signer]'s secret key.
    pub fn sign(&self, digest: impl Into<[u8; 32]>) -> Signature {
        let secp = Secp256k1::new();
        let signature = secp.sign_ecdsa_recoverable(&Message::from_digest(digest.into()), &self.sk);
        let (recovery_id, data) = signature.serialize_compact();
        Signature {
            r: U256::try_from_be_slice(&data[..32]).unwrap(),
            s: U256::try_from_be_slice(&data[32..64]).unwrap(),
            recovery_id: recovery_id.to_i32(),
        }
    }
}
