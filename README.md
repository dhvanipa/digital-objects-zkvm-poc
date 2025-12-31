# digital-objects-zkvm-poc

A proof of concept of digital objects inspired by [digital-objects-e2e-poc](https://github.com/0xPARC/digital-objects-e2e-poc), but implemented with a zero-knowledge virtual machine (zkVM) instead of the [POD](https://github.com/0xPARC/pod2) framework.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1 zkVM](https://docs.succinct.xyz/docs/sp1/getting-started/install)

## Running the Project

```
RUST_LOG=info cargo run --release --bin craftlib
```
