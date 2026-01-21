//! A commit object program

use risc0_zkvm::guest::env;

use pow_program::{PowIn, PowOut};
use sha2::{Digest, Sha256};

fn main() {
    let inp = env::read::<PowIn>();

    let mut cur: [u8; 32] = hex::decode(&inp.input)
        .expect("valid hex input")
        .try_into()
        .expect("32 bytes");
    for _ in 0..inp.n_iters {
        let mut h = Sha256::new();
        h.update(cur);
        cur = h.finalize().into();
    }

    env::commit(&PowOut {
        n_iters: inp.n_iters,
        input: inp.input,
        output: hex::encode(cur),
    });
}
