use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use crate::{
    Block, BlockHeader, Blockchain, SignedBlockHeader, SignedTransaction, Signer, Transaction,
    BLOCK_PERIOD,
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
    /// Interval of time between blocks.
    block_timer: tokio::time::Interval,
}

impl Sequencer {
    /// Creates a new permissioned [Sequencer].
    pub fn new(signer: impl Into<Signer>) -> Self {
        Sequencer {
            signer: signer.into(),
            blockchain: Blockchain::default(),
            transactions_pool: vec![],
            withdrawals_pool: vec![],
            block_timer: tokio::time::interval(BLOCK_PERIOD),
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
        // Record the time the latest block time.
        let block_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Construct the block header.
        let header = BlockHeader {
            sequencer: self.signer.address,
            number: self.blockchain.height(),
            timestamp: block_time,
            parent_digest: self.blockchain.head().map(|b| b.hash()),
            withdrawals_root: format!("{:x}", self.blockchain.withdrawals_tree.root()),
            transactions_root: format!("{:x}", self.blockchain.transactions_tree.root()),
        };

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
    pub fn head(&self) -> Option<Block> {
        self.blockchain.head()
    }
}

/// A shared, thread-safe [Sequencer].
#[derive(Clone)]
pub struct ArcSequencer(Arc<Mutex<Sequencer>>);

impl ArcSequencer {
    /// Creates a new [ArcSequencer] from a [Sequencer].
    pub fn new(signer: impl Into<Signer>) -> Self {
        ArcSequencer(Arc::new(Mutex::new(Sequencer::new(signer))))
    }

    /// Locks the sequencer for exclusive access.
    pub async fn lock(&self) -> std::sync::MutexGuard<'_, Sequencer> {
        self.0.lock().unwrap()
    }
}

/// A future that infinitely seals blocks at a fixed period.
impl Future for ArcSequencer {
    type Output = Block;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut sequencer = self.get_mut().0.lock().unwrap();
        if sequencer.block_timer.poll_tick(cx).is_ready() {
            Poll::Ready(sequencer.seal())
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::task::JoinSet;

    use super::*;

    #[tokio::test]
    async fn test_sequencer() {
        // Create a sequencer.
        let signer = Signer::random();
        let sequencer = ArcSequencer::new(signer);
        let mut sequencer = sequencer.lock().await;

        // Add a transaction to the sequencer.
        let transaction = SignedTransaction::new(
            Transaction::dynamic(sequencer.signer.address, 100, 1),
            &sequencer.signer,
        );
        sequencer.add_transaction(transaction.clone());

        // Seal the block.
        let block = sequencer.seal();

        // Validate the block.
        assert_eq!(block.transactions.len(), 1);
        assert_eq!(block.transactions[0], transaction);
        assert_eq!(sequencer.head().unwrap(), block);
        assert!(block.verify());
    }

    #[tokio::test]
    async fn test_concurrent_sequencers() {
        // Create a sequencer.
        let signer = Signer::random();
        let sequencer = ArcSequencer::new(signer);

        // Spawn concurrent tasks to add transactions and seal blocks.
        let mut set = JoinSet::new();
        for _ in 0..10 {
            let sequencer = sequencer.clone();
            let task = tokio::task::spawn(async move {
                let mut sequencer = sequencer.lock().await;
                let transaction = SignedTransaction::new(
                    Transaction::dynamic(sequencer.signer.address, 100, 1),
                    &sequencer.signer,
                );
                sequencer.add_transaction(transaction);
                sequencer.seal()
            });
            set.spawn(task);
        }

        // Join the tasks and verify the blocks.
        while let Some(r) = set.join_next().await {
            assert!(r.unwrap().unwrap().verify());
        }
    }
}
