mod transaction;
use transaction::WithdrawalTxData;
pub use transaction::{SignedTransaction, Transaction};

mod signer;
use signer::Signature;
pub use signer::Signer;

mod block;
pub use block::{Block, BlockHeader, SignedBlockHeader};

mod sequencer;
pub use sequencer::ArcSequencer;

mod blockchain;
use blockchain::Blockchain;

mod address;
pub use address::Address;

pub const BLOCK_PERIOD_MILLIS: u64 = 2000;
pub const CHAIN_ID: u64 = 83479;
