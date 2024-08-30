use alloy_primitives::{keccak256, B256};
use secp256k1::{Message, Secp256k1};
use serde::{Deserialize, Serialize};

use crate::signer::{Signature, Signer};
use crate::{Address, CHAIN_ID};

/// A transaction header containing metadata about the transaction.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TransactionHeader {
    /// The identifier of the chain on which the transaction was intended to be executed.
    chain_id: u64,
    /// The address of the sender of the transaction.
    sender: Address,
    /// The address of the recipient of the transaction.
    recipient: Address,
    /// The amount of value transferred by the transaction.
    amount: u64,
}

/// A dynamic transaction containing a transaction header and dynamic fee data.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct DynamicTxData {
    /// The transaction header.
    header: TransactionHeader,
    /// The maximum fee per gas that the sender is willing to pay.
    max_fee_per_gas: u64,
    /// The maximum priority fee per gas that the sender is willing to pay.
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
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct WithdrawalTxData {
    /// The transaction header.
    header: TransactionHeader,
    /// The destination chain of the withdrawal.
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
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
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
                chain_id: CHAIN_ID,
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
                chain_id: CHAIN_ID,
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
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
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
        let address = Address::from(pk);
        secp.verify_ecdsa(&msg, &(&self.signature).into(), &pk)
            .is_ok()
            && self.transaction.sender() == address
    }
}
