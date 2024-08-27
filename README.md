# The Nolemma Rollup

A blazingly fast rollup that solves the infamous trilemma by leveraging a revolutionary architecture that is:
* Centralized;
* Ephemeral; and
* Useless.

## Cryptographic Operations

### Elliptic Curve Digital Signatures and Keccak Hashes

...

### Incremental Merkle Trees

...

## Usage

You can launch a local Nolemma sequencer and send transactions to it through the following command:
```sh
cargo run -p script
```

The sequencer will run in its own process, sealing blocks at a fixed period and accepting requests to submit transactions.

A separate process will regularly send random transactions to the sequencer and verify resulting blocks.
