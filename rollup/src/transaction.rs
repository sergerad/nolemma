use alloy_primitives::{keccak256, Address, B256};
use secp256k1::{Message, Secp256k1};
use serde::{Deserialize, Serialize};

use crate::signer::{Signature, Signer};

/// A transaction header containing metadata about the transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionHeader {
    chain_id: u64,
    sender: Address,
    recipient: Address,
    amount: u64,
}

/// A dynamic transaction containing a transaction header and dynamic fee data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DynamicTxData {
    header: TransactionHeader,
    max_fee_per_gas: u64,
    max_priority_fee_per_gas: u64,
}

impl DynamicTxData {
    /// Computes the hash of the dynamic transaction.
    pub fn hash(&self) -> B256 {
        let bytes = bincode::serialize(self).unwrap();
        keccak256(bytes)
    }
}

/// A withdrawal transaction containing a transaction header and destination.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WithdrawalTxData {
    header: TransactionHeader,
    dest_chain: u64,
}

impl WithdrawalTxData {
    /// Computes the hash of the withdrawal transaction.
    pub fn hash(&self) -> B256 {
        let bytes = bincode::serialize(self).unwrap();
        keccak256(bytes)
    }
}

/// A transaction containing either dynamic or withdrawal transaction data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Transaction {
    Dynamic(DynamicTxData),
    Withdrawal(WithdrawalTxData),
    // Legacy ...
    // Blob ...
}

impl Transaction {
    /// Creates a new dynamic transaction.
    pub fn dynamic(sender: Address, amount: u64) -> Self {
        Transaction::Dynamic(DynamicTxData {
            header: TransactionHeader {
                chain_id: 1,
                sender,
                recipient: Address::random(),
                amount,
            },
            max_fee_per_gas: 0,
            max_priority_fee_per_gas: 0,
        })
    }

    /// Creates a new withdrawal transaction.
    pub fn withdrawal(sender: Address, amount: u64, dest_chain: u64) -> Self {
        Transaction::Withdrawal(WithdrawalTxData {
            header: TransactionHeader {
                chain_id: 1,
                sender,
                recipient: sender,
                amount,
            },
            dest_chain,
        })
    }

    /// Computes the hash of the transaction.
    pub fn hash(&self) -> B256 {
        match self {
            Transaction::Dynamic(tx) => tx.hash(),
            Transaction::Withdrawal(tx) => tx.hash(),
        }
    }

    /// Returns the sender of the transaction.
    pub fn sender(&self) -> Address {
        match self {
            Transaction::Dynamic(tx) => tx.header.sender,
            Transaction::Withdrawal(tx) => tx.header.sender,
        }
    }
}

/// A signed transaction containing a transaction and signature.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Signature,
}

impl SignedTransaction {
    /// Creates a new signed transaction.
    pub fn new(transaction: Transaction, signer: &Signer) -> SignedTransaction {
        let signature = signer.sign(transaction.hash());
        SignedTransaction {
            transaction,
            signature,
        }
    }

    /// Verifies the signature of the [SignedTransaction] is valid and that it matches
    /// the address of the sender specified in the [TransactionHeader].
    pub fn verify(&self) -> bool {
        let secp = Secp256k1::new();
        let msg = Message::from_digest(self.transaction.hash().into());
        let pk = secp.recover_ecdsa(&msg, &(&self.signature).into()).unwrap();
        let digest = keccak256(&pk.serialize_uncompressed()[1..]);
        let address = Address::from_slice(&digest[12..]);
        secp.verify_ecdsa(&msg, &(&self.signature).into(), &pk)
            .is_ok()
            && self.transaction.sender() == address
    }
}
