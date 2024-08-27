use crate::{
    Block, BlockHeader, Blockchain, SignedBlockHeader, SignedTransaction, Signer, Transaction,
};

/// Permissioned entity responsible for maintaining the canonical [Blockchain].
/// Receives transactions directly and seals them into blocks.
pub struct Sequencer {
    signer: Signer,
    blockchain: Blockchain,
    pool: Vec<SignedTransaction>,
    withdrawals: Vec<SignedTransaction>,
}

impl Sequencer {
    /// Creates a new permissioned [Sequencer].
    pub fn new(signer: impl Into<Signer>) -> Self {
        Sequencer {
            signer: signer.into(),
            blockchain: Blockchain::default(),
            pool: vec![],
            withdrawals: vec![],
        }
    }

    /// Adds a transaction to the pool to be included in the next block.
    pub fn add_transaction(&mut self, transaction: SignedTransaction) {
        match &transaction.transaction {
            Transaction::Withdrawal(tx) => {
                self.blockchain.withdraw(tx);
                self.withdrawals.push(transaction);
            }
            Transaction::Dynamic(tx) => {
                self.blockchain.transact(tx);
                self.pool.push(transaction);
            }
        }
    }

    /// Creates the latest canonical block and signs.
    /// Transaction pools are cleared during this process.
    pub fn seal(&mut self) -> Block {
        // Construct the block header.
        let header = BlockHeader::new(
            self.signer.address,
            self.blockchain.height(),
            self.blockchain.head().map(|b| b.hash().to_string()),
            format!("{:x}", self.blockchain.withdrawals_tree.root()),
            format!("{:x}", self.blockchain.transactions_tree.root()),
        );

        // Drain the transaction pools and construct the block.
        let block = Block::new(
            SignedBlockHeader::new(header, &self.signer),
            self.pool
                .drain(..)
                .chain(self.withdrawals.drain(..))
                .collect(),
        );
        self.blockchain.push(block.clone());
        block
    }

    /// Returns the head block of the blockchain.
    pub fn head(&self) -> Option<&Block> {
        self.blockchain.head()
    }
}
