#[macro_use]
extern crate rocket;

use rocket::State;
use rocket::{serde::json::Json, Config};
use rollup::{ArcSequencer, SignedTransaction};
use serde_json::{json, Value};

/// Accepts a transaction and adds it to the respective transaction pools.
#[post("/", data = "<payload>")]
async fn submit(sequencer: &State<ArcSequencer>, payload: Json<SignedTransaction>) -> Value {
    // Extract the transaction from the payload.
    let transaction = payload.into_inner();
    let tx_digest = transaction.transaction.hash();

    // Add the transaction to the pool.
    let mut sequencer = sequencer.lock().await;
    sequencer.add_transaction(transaction);

    // Respond with the transaction digest.
    json!({ "tx_digest": tx_digest.to_string() })
}

/// Returns the head block of the blockchain.
#[get("/")]
async fn head(sequencer: &State<ArcSequencer>) -> Value {
    // Retrieve the head block from the sequencer and return it.
    let sequencer = sequencer.lock().await;
    let head = sequencer.head();
    json!(head)
}

/// Infinitely seals blocks at a fixed period.
async fn seal_blocks_loop(sequencer: ArcSequencer) {
    tokio::task::spawn(async move {
        loop {
            let block = sequencer.clone().await;
            println!("Sealed block: {:?} {}", block.number(), block.hash());
        }
    });
}

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    // Set up sequencer.
    let sk = std::env::var("KEY").unwrap();
    let sequencer = ArcSequencer::new(sk.as_str());

    // Spawn block producing sequencer task.
    seal_blocks_loop(sequencer.clone()).await;

    // Launch the HTTP server.
    let config = Config {
        log_level: rocket::config::LogLevel::Critical,
        ..Config::debug_default()
    };
    rocket::build()
        .configure(config)
        .mount("/", routes![submit, head])
        .manage(sequencer)
}
