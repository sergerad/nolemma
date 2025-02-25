use crate::{transaction::DynamicTxData, Block, WithdrawalTxData};

/// A blockchain containing a list of blocks and an incremental Merkle tree of withdrawals.
pub struct Blockchain {
    /// The chain of blocks in the blockchain.
    pub(crate) blocks: Vec<Block>,
    /// The incremental Merkle tree of withdrawals.
    pub(crate) withdrawals_tree: imt::Tree<sha2::Sha256>,
    /// The incremental Merkle tree of transactions.
    pub(crate) transactions_tree: imt::Tree<sha2::Sha256>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Blockchain {
            blocks: vec![],
            withdrawals_tree: imt::Builder::default().build().unwrap(),
            transactions_tree: imt::Builder::default().build().unwrap(),
        }
    }
}

impl Blockchain {
    /// Returns the head block of the blockchain.
    pub fn head(&self) -> Option<Block> {
        self.blocks.last().cloned()
    }

    /// Returns the height of the blockchain.
    pub(crate) fn height(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// Pushes a block onto the blockchain.
    pub(crate) fn push(&mut self, block: Block) {
        self.blocks.push(block);
    }

    /// Appends a withdrawal transaction to the respective incremental Merkle tree.
    pub(crate) fn withdraw(&mut self, tx: &WithdrawalTxData) {
        let hash = tx.hash();
        self.withdrawals_tree.add_leaf(hash).unwrap();
    }

    /// Appends a dynamic transaction to the respective incremental Merkle tree.
    pub(crate) fn transact(&mut self, tx: &DynamicTxData) {
        let hash = tx.hash();
        self.transactions_tree.add_leaf(hash).unwrap();
    }
}
