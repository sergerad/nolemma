use rollup::{Block, SignedTransaction, Signer, Transaction, BLOCK_PERIOD_MILLIS};
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
async fn send_transaction(tx: SignedTransaction) {
    if let Err(e) = reqwest::Client::new()
        .post(&format!("http://{}/", SEQUENCER_URL))
        .json(&tx)
        .send()
        .await
    {
        println!("Error sending transaction: {:?}", e);
    }
}

/// Infinitely sends transactions to the sequencer.
async fn tx_loop() {
    // Wait for the sequencer to start.
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    for i in 0.. {
        // Send a deposit transaction.
        let signer = Signer::random();
        let transaction = Transaction::dynamic(signer.address, i);
        let signed = SignedTransaction::new(transaction, &signer);
        send_transaction(signed).await;

        // Send a withdrawal transaction.
        let dest_chain = 2u64;
        let transaction = Transaction::withdrawal(signer.address, i, dest_chain);
        let signed = SignedTransaction::new(transaction, &signer);
        send_transaction(signed).await;

        // Wait before sending the next transactions.
        tokio::time::sleep(tokio::time::Duration::from_millis(BLOCK_PERIOD_MILLIS / 4)).await;
    }
}

async fn head_loop() {
    // Wait for some blocks.
    tokio::time::sleep(tokio::time::Duration::from_millis(BLOCK_PERIOD_MILLIS * 2)).await;
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
        tokio::time::sleep(tokio::time::Duration::from_millis(BLOCK_PERIOD_MILLIS)).await;
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
