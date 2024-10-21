use std::sync::Arc;

use log::info;
use tokio::sync::Mutex;

use crate::{
    Block, BlockHeader, Blockchain, SignedBlockHeader, SignedTransaction, Signer, Transaction,
    BLOCK_PERIOD,
};

pub struct TransactionSubmitter {
    transactions_pool: Arc<Mutex<Vec<SignedTransaction>>>,
}

impl TransactionSubmitter {
    pub fn new(transactions_pool: Arc<Mutex<Vec<SignedTransaction>>>) -> Self {
        TransactionSubmitter { transactions_pool }
    }

    pub async fn submit(&self, transaction: SignedTransaction) {
        let transactions_pool = self.transactions_pool.clone();
        transactions_pool.lock().await.push(transaction);
    }
}

/// Permissioned entity responsible for maintaining the canonical [Blockchain].
/// Receives transactions directly and seals them into blocks.
pub struct Sequencer {
    /// The sequencer's signer used to sign blocks.
    signer: Signer,
    /// The blockchain maintained by the sequencer.
    blockchain: Arc<Mutex<Blockchain>>,
    /// The pool of transactions to be included in the next block.
    transactions_pool: Arc<Mutex<Vec<SignedTransaction>>>,
    /// The pool of withdrawal transactions to be included in the next block.
    withdrawals_pool: Vec<SignedTransaction>,
    /// Interval of time between blocks.
    block_timer: tokio::time::Interval,
}

impl Sequencer {
    /// Creates a new permissioned [Sequencer].
    pub fn new(
        signer: impl Into<Signer>,
        transactions_pool: Arc<Mutex<Vec<SignedTransaction>>>,
        blockchain: Arc<Mutex<Blockchain>>,
    ) -> Self {
        Sequencer {
            signer: signer.into(),
            transactions_pool,
            blockchain,
            withdrawals_pool: vec![],
            block_timer: tokio::time::interval(BLOCK_PERIOD),
        }
    }

    /// Runs the sequencer's main loop.
    pub async fn run(&mut self) {
        loop {
            self.block_timer.tick().await;
            let block = self.seal().await;
            info!("Sealed block: {:?}", block);
        }
    }

    /// Adds a transaction to the pool to be included in the next block.
    pub async fn add_transaction(&mut self, transaction: SignedTransaction) {
        match &transaction.transaction {
            Transaction::Withdrawal(tx) => {
                self.blockchain.lock().await.withdraw(tx);
                self.withdrawals_pool.push(transaction);
            }
            Transaction::Dynamic(tx) => {
                self.blockchain.lock().await.transact(tx);
                self.transactions_pool.lock().await.push(transaction);
            }
        }
    }

    /// Creates the latest canonical block and signs.
    /// Transaction pools are cleared during this process.
    pub async fn seal(&mut self) -> Block {
        // Record the time the latest block time.
        let block_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Construct the block header.
        let mut chain = self.blockchain.lock().await;
        let header = BlockHeader {
            sequencer: self.signer.address,
            number: chain.height(),
            timestamp: block_time,
            parent_digest: chain.head().map(|b| b.hash()),
            withdrawals_root: format!("{:x}", chain.withdrawals_tree.root()),
            transactions_root: format!("{:x}", chain.transactions_tree.root()),
        };

        // Drain the transaction pools and construct the block.
        let block = Block::new(
            SignedBlockHeader::new(header, &self.signer),
            self.transactions_pool
                .lock()
                .await
                .drain(..)
                .chain(self.withdrawals_pool.drain(..))
                .collect(),
        );
        chain.push(block.clone());
        block
    }

    /// Returns the head block of the blockchain.
    pub async fn head(&self) -> Option<Block> {
        self.blockchain.lock().await.head()
    }
}
