# digital-objects-zkvm-poc

A proof of concept of digital objects inspired by [digital-objects-e2e-poc](https://github.com/0xPARC/digital-objects-e2e-poc), but implemented with a zero-knowledge virtual machine (zkVM) instead of the [POD](https://github.com/0xPARC/pod2) framework.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1 zkVM](https://docs.succinct.xyz/docs/sp1/getting-started/install)

## Docs

- [Digital Objects Explained](https://hackmd.io/@dhvanipa/Sk_n-cHVZx)
- [Digital Objects via zkVM](https://hackmd.io/@dhvanipa/Bk7Fc5r4Wl)

## Running the Project

0. Create an `.env` from `.env.example` and set the private key to a wallet that has funds on Ethereum Sepolia.

1. Run the synchronizer which will continously print the current global state.

```
RUST_LOG=info cargo run --release --bin synchronizer
```

2. Craft digital objects

This command will craft 1 wood, 1 stone, and 1 axe made up of those wood and stone. They will be saved in `objects/`.

```
RUST_LOG=info cargo run --release --bin craftlib
```

3. Commit digital objects

First commit the wood.

```
RUST_LOG=info cargo run --release --bin commitlib objects/wood_1.json
```

Then the stone.

```
RUST_LOG=info cargo run --release --bin commitlib objects/stone_1.json
```

Then the axe.

```
RUST_LOG=info cargo run --release --bin commitlib objects/axe_1.json
```

At each commitment, you should see the global state update.
