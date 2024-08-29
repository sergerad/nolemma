use alloy_primitives::{keccak256, B256};
use secp256k1::{Message, Secp256k1};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Address, Signature, SignedTransaction, Signer};

/// A block header containing metadata about the block.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    /// The address of the sequencer that sealed the block.
    pub sequencer: Address,
    /// The number of the block.
    pub number: u64,
    /// The timestamp at the time the block was sealed.
    pub timestamp: u64,
    /// The hash of the parent block. None if this is the genesis block.
    pub parent_digest: Option<String>,
    /// The root digest of the withdrawals Merkle tree.
    pub withdrawals_root: String,
    /// The root digest of the transactions Merkle tree.
    pub transactions_root: String,
}

impl BlockHeader {
    /// Creates a new block header.
    pub fn new(
        sequencer: Address,
        number: u64,
        parent_digest: Option<String>,
        withdrawals_root: String,
        transactions_root: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        BlockHeader {
            sequencer,
            number,
            timestamp,
            parent_digest,
            withdrawals_root,
            transactions_root,
        }
    }

    /// Computes the hash of the block header.
    pub fn hash(&self) -> B256 {
        let bytes = bincode::serialize(self).unwrap();
        keccak256(bytes)
    }
}

/// A signed block header containing a block header and a signature.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedBlockHeader {
    header: BlockHeader,
    signature: Signature,
}

impl SignedBlockHeader {
    /// Creates a new signed block header with the given header and signer.
    pub fn new(header: BlockHeader, signer: &Signer) -> Self {
        let signature = signer.sign(header.hash());
        Self { header, signature }
    }
}

/// A block containing a header and a list of transactions.
#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub(crate) signed: SignedBlockHeader,
    transactions: Vec<SignedTransaction>,
}

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block")
            .field("signed", &self.signed)
            .field(
                "transactions",
                &self
                    .transactions
                    .iter()
                    .map(|tx| tx.transaction.hash())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Block {
    /// Creates a new block with the given header and transactions.
    pub fn new(header: SignedBlockHeader, transactions: Vec<SignedTransaction>) -> Self {
        Block {
            signed: header,
            transactions,
        }
    }

    /// Computes the hash of the block.
    pub fn hash(&self) -> B256 {
        let bytes = bincode::serialize(&self.signed.header).unwrap();
        keccak256(bytes)
    }

    /// Verifies the signature of the [Block] is valid and that it matches
    /// the sequencer address specified in the [SignedBlockHeader].
    pub fn verify(&self) -> bool {
        let secp = Secp256k1::new();
        let msg = Message::from_digest(self.hash().into());
        let pk = secp
            .recover_ecdsa(&msg, &(&self.signed.signature).into())
            .unwrap();
        let address = Address::from(pk);
        secp.verify_ecdsa(&msg, &(&self.signed.signature).into(), &pk)
            .is_ok()
            && self.signed.header.sequencer == address
    }

    /// Returns the number of the block.
    pub fn number(&self) -> u64 {
        self.signed.header.number
    }
}
