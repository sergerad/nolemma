# Nolemma

Nolemma is a blazingly fast rollup, leveraging a revolutionary architecture that is:
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

The output should looking something like this:
```sh
Block 0 verified: true
Block {
    signed: SignedBlockHeader {
        header: BlockHeader {
            sequencer: 0xecfd4745665c6c5b72b2bc558e80a207283f7091,
            number: 0,
            timestamp: 1724918355,
            parent_digest: None,
            withdrawals_root: "f62a6ed7b5baffb33a21235cf528bb86c37ca38940539e92f6b71f5c26d682b1",
            transactions_root: "4a9cb97beeb462e562f6d52c54b11172c6dbcafc80de1d46c104a0f860daa7fb",
        },
        signature: Signature {
            r: 37593562742579918054880423626579338602546470885109318820256556408931428206851,
            s: 37641522295278184479778267846185146638534229227713144928721563320698850950022,
            recovery_id: 0,
        },
    },
    transactions: [
        0x3d86fd1ad620a6fbabfd37386bd5d75e16992695fff9601e1846a2a1e6f40989,
        0xd3a71716d8dc3b2a790f5889da122329b0519a602c46f7dd5f2d0727bd95b7cc,
        0x8de71c98577f73742eb0df55ed8721d886bb2f89645830e0e4f7c2ca8966a0df,
        0x0dfd72acaee1a8e1d71f5d773894ec907c69da09800965e7cb834744d8ef6bb2,
        0x25de9af1a9fc56c3c702f7c002a15d06de2de0fc675b49017a3f493a0862a925,
        0x608b7d34d41e7e7bb76061b41bd49bce70b65d79fea8fa1eb328304ba8bc1afd,
        0x11bc6f206b6c55fa6de241a7740bfcbe85e01c7cabc47552d5345cfd8d4083b5,
        0xbc827cb31a81a12874d115042861918d259088fd86927ae89ae7ec2da7e9bffe,
    ],
}
```

