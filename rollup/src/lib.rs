mod transaction;
use std::time::Duration;

use transaction::WithdrawalTxData;
pub use transaction::{SignedTransaction, Transaction};

mod signer;
use signer::Signature;
pub use signer::Signer;

mod block;
pub use block::{Block, BlockHeader, SignedBlockHeader};

mod sequencer;
pub use sequencer::{Sequencer, TransactionSubmitter};

mod blockchain;
pub use blockchain::Blockchain;

mod address;
pub use address::Address;

pub const BLOCK_PERIOD: Duration = Duration::from_secs(2);
pub const CHAIN_ID: u64 = 83479;
