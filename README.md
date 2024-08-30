# Nolemma

Nolemma is a blazingly fast rollup, leveraging a revolutionary architecture that is:
* Centralized;
* Ephemeral; and
* Useless.

Because of this design, Nolemma is capable of achieving near real-time block production.

## Repository Structure

The workspace contains the following:
* `rollup` library crate for all core types and functionality;
* `sequencer` binary crate for running a sequencer; and
* `script` binary crate for bootstrapping a local sequencer, sending transactions, and validating sealed blocks.

## Usage

You can launch a local Nolemma sequencer and send transactions to it through the following command:
```sh
cargo run -p script
```

The sequencer will run in its own process, sealing blocks at a fixed period and accepting requests to submit transactions.

A separate process will regularly send signed transactions to the sequencer and verify resulting blocks.

The output should looking something like this:
```sh
Sealed block: 4 0xc33d9a6354174872aa21bf4b4f0cf5ee20e63559f08728cbfc4702a159c060c1
Block 4 verified: true
Block {
    signed: SignedBlockHeader {
        header: BlockHeader {
            sequencer: Address(
                0xfdf3d23e0a93f940ecae4fab8ee89fa87ade3345,
            ),
            number: 4,
            timestamp: 1724971199,
            parent_digest: Some(
                "0x030403926131dc82f2aabbc98e7f49a2afa3bb8862a58f45dbff5c31f423aa6b",
            ),
            withdrawals_root: "f6156eb252cf9385b32bbf9d67c98743a54069ddb82711347bf9de98a4e84a1a",
            transactions_root: "b7bf4ee47ef33df507312849ea2c511d8b1cac896c9ad4632ac84db27c29be96",
        },
        signature: Signature {
            r: 99516742611579300247107593222426462760699249586140514477002059713627822811124,
            s: 52840667303596146942485583793658493738843174190414595374809724448462630931548,
            v: 0,
        },
    },
    transactions: [
        0x0917464362e4c08429bbdb02c80d2e8fcd5c1e5d08329f91e102af8ace99e219,
        0xa5d5f4f8df5b62f4ebb1d60104c61b9cd820d4019862ab7d56578502bfc2a92d,
        0xd2ae8e2c10b805591fc92339db73969ba1691b250c979b047e272ac54bfa9583,
        0x1427d8a64d6b39529afcd2c8a1626cf2b033659a4a2ecf3ff77d544233686495,
        0xa79616ff5ec9ce53e1380ab4de17e21dd6f033debc848c83d4cf6397893d37d9,
        0x9511ea9207d1be4a2a01098494688711794e31f4f445f545728ea9e46dc5bf8f,
        0xaa1dbb27eac9340e523d8a06c2e103f49c822e2020c7f2d45d03b50c0183d5e3,
        0xc94be521f7c4fad1fa52dede7b3a2e222390c3a06c9596bf4287dbef8f7a4653,
    ],
}
```

## Protocol Design

The system is a toy protocol and it is only very partially implemented.

The key cryptographic primitives used by the protocol are the following:
* ECDSA secp256k1 for signatures of both transactions and blocks;
* Keccak256 for all hashing purposes including ECDSA, content-addressable IDs (transaction and block hashes), as well as construction of Addresses (last 20 bytes of the hash); and
* Incremental Merkle trees for withdrawal transactions which allows for L2->L1 transfers via Merkle proofs.

### Sequencing

There is a single, permissioned sequencer. It produces blocks at a fixed period. Blocks are hashed with Keccak256 and signed with secp256k1 ECDSA.

Block headers contain the following:
* Number
* Timestamp
* Parent block digest
* Sequencer's address
* Withdrawals Merkle tree root
* Transactions Merkle tree root
* Sequencer's signature of the bock

The remainder of block data is consumed by transactions that were sealed into the block.

### Transaction Types and Lifecycle

Nolemma currently supports two types of transactions - dynamic and withdrawal.

Dynamic transctions are simply EIP-1559 style transactions.

Withdrawals are a custom transaction type used for withdrawing funds from the L2.

L2 transaction finality depends on verification of validity proofs on L1. This feature is not yet implemented.

### Withdrawals

Withdrawal transctions are a custom type of transaction supported by Nolemma.

When withdrawal transactions are sealed into blocks, they are added to an incremental Merkle tree. This tree is treated as an "exit tree" for withdrawals. The L1 smart contract relies on Merkle proofs of withdrawal transactions against the root of the tree in order to execute the final step of a withdrawal - its exit on L1.
