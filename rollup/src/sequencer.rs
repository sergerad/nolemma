use crate::{
    Block, BlockHeader, Blockchain, SignedBlockHeader, SignedTransaction, Signer, Transaction,
};

/// Permissioned entity responsible for maintaining the canonical [Blockchain].
/// Receives transactions directly and seals them into blocks.
pub struct Sequencer {
    /// The sequencer's signer used to sign blocks.
    signer: Signer,
    /// The blockchain maintained by the sequencer.
    blockchain: Blockchain,
    /// The pool of transactions to be included in the next block.
    transactions_pool: Vec<SignedTransaction>,
    /// The pool of withdrawal transactions to be included in the next block.
    withdrawals_pool: Vec<SignedTransaction>,
}

impl Sequencer {
    /// Creates a new permissioned [Sequencer].
    pub fn new(signer: impl Into<Signer>) -> Self {
        Sequencer {
            signer: signer.into(),
            blockchain: Blockchain::default(),
            transactions_pool: vec![],
            withdrawals_pool: vec![],
        }
    }

    /// Adds a transaction to the pool to be included in the next block.
    pub fn add_transaction(&mut self, transaction: SignedTransaction) {
        match &transaction.transaction {
            Transaction::Withdrawal(tx) => {
                self.blockchain.withdraw(tx);
                self.withdrawals_pool.push(transaction);
            }
            Transaction::Dynamic(tx) => {
                self.blockchain.transact(tx);
                self.transactions_pool.push(transaction);
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
            self.transactions_pool
                .drain(..)
                .chain(self.withdrawals_pool.drain(..))
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
