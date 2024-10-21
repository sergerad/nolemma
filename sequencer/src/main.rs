#[macro_use]
extern crate rocket;

use std::sync::Arc;

use rocket::State;
use rocket::{serde::json::Json, Config};
use rollup::{Blockchain, Sequencer, SignedTransaction, TransactionSubmitter};
use serde_json::{json, Value};
use tokio::sync::Mutex;

/// Accepts a transaction and adds it to the respective transaction pools.
#[post("/", data = "<payload>")]
async fn submit(
    submitter: &State<TransactionSubmitter>,
    payload: Json<SignedTransaction>,
) -> Value {
    // Extract the transaction from the payload.
    let transaction = payload.into_inner();
    let tx_digest = transaction.transaction.hash();

    // Add the transaction to the pool.
    submitter.submit(transaction).await;

    // Respond with the transaction digest.
    json!({ "tx_digest": tx_digest.to_string() })
}

/// Returns the head block of the blockchain.
#[get("/")]
async fn head(chain: &State<Arc<Mutex<Blockchain>>>) -> Value {
    // Retrieve the head block from the sequencer and return it.
    let head = chain.lock().await.head();
    json!(head)
}

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    env_logger::init();
    // Set up sequencer.
    let sk = std::env::var("KEY").unwrap();
    let pool = Arc::new(tokio::sync::Mutex::new(vec![]));
    let chain = Arc::new(tokio::sync::Mutex::new(Blockchain::default()));
    let mut sequencer = Sequencer::new(sk.as_str(), pool.clone(), chain.clone());
    let submitter = TransactionSubmitter::new(pool);

    // Spawn block producing sequencer task.
    tokio::task::spawn(async move {
        sequencer.run().await;
    });

    // Launch the HTTP server.
    let config = Config {
        log_level: rocket::config::LogLevel::Critical,
        ..Config::debug_default()
    };
    rocket::build()
        .configure(config)
        .mount("/", routes![submit, head])
        .manage(submitter)
        .manage(chain)
}
