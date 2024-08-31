use rollup::{Block, SignedTransaction, Signer, Transaction, BLOCK_PERIOD};
use secp256k1::SecretKey;
use tokio::process::Command;

/// Specifies the anticipated URL that the sequencer will listen on.
const SEQUENCER_URL: &str = "127.0.0.1:8000";

/// Runs the sequencer process and blocks on it's completion.
async fn run_sequencer(sk: SecretKey) {
    let mut sequencer = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("sequencer")
        .arg("--")
        .env("KEY", hex::encode(sk.secret_bytes()))
        .kill_on_drop(true)
        .spawn()
        .expect("Failed to start sequencer process");
    let _ = sequencer
        .wait()
        .await
        .expect("Failure while waiting for sequencer process");
}

/// Sends the provided transaction to the sequencer and waits for the response.
async fn send_transaction(tx: SignedTransaction) -> Result<reqwest::Response, reqwest::Error> {
    reqwest::Client::new()
        .post(&format!("http://{}/", SEQUENCER_URL))
        .json(&tx)
        .send()
        .await
}

/// Sleeps for a short period of time and prints an error message.
async fn handle_request_err(e: reqwest::Error) {
    if e.is_connect() {
        println!("Sequencer not available yet, retrying...");
    } else {
        println!("Error sending transaction: {:?}", e);
    }
    tokio::time::sleep(BLOCK_PERIOD).await;
}

/// Infinitely sends transactions to the sequencer.
async fn tx_loop() {
    // Wait for the sequencer to start.
    tokio::time::sleep(BLOCK_PERIOD).await;
    for i in 0.. {
        // Send a deposit transaction.
        let signer = Signer::random();
        let transaction = Transaction::dynamic(signer.address, i, i);
        let signed = SignedTransaction::new(transaction, &signer);
        if let Err(e) = send_transaction(signed).await {
            handle_request_err(e).await;
            continue;
        }

        // Send a withdrawal transaction.
        let dest_chain = 1u64;
        let transaction = Transaction::withdrawal(signer.address, i, dest_chain, i);
        let signed = SignedTransaction::new(transaction, &signer);
        if let Err(e) = send_transaction(signed).await {
            handle_request_err(e).await;
            continue;
        }

        // Wait before sending the next transactions.
        tokio::time::sleep(BLOCK_PERIOD / 4).await;
    }
}

async fn head_loop() {
    // Wait for some blocks.
    tokio::time::sleep(BLOCK_PERIOD * 2).await;
    loop {
        // Get the head block from the sequencer.
        match reqwest::get(&format!("http://{}/", SEQUENCER_URL)).await {
            // Parse the head block and print it.
            Ok(res) => match res.json::<Option<Block>>().await {
                Ok(Some(head)) => {
                    println!("Block {} verified: {:?}", head.number(), head.verify());
                    println!("{:#?}", head);
                }
                Ok(None) => {
                    println!("No blocks yet");
                }
                Err(e) => {
                    println!("Error parsing head block: {:?}", e);
                }
            },
            Err(e) => {
                println!("Error getting head block: {:?}", e);
            }
        }
        tokio::time::sleep(BLOCK_PERIOD).await;
    }
}

#[tokio::main]
async fn main() {
    // Random permissioned sequencer.
    let Signer { sk, .. } = Signer::random();

    // Run the sequencer.
    tokio::spawn(async move {
        run_sequencer(sk).await;
    });

    // Continuously check the head block.
    tokio::spawn(async move {
        head_loop().await;
    });
    // Send transactions to the sequencer.
    tx_loop().await;
}
