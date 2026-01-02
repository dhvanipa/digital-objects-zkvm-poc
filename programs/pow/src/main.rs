//! A commit object program

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use pow_program::{PowIn, PowOut};
use sha2::{Digest, Sha256};

pub fn main() {
    let inp = sp1_zkvm::io::read::<PowIn>();

    let mut cur: [u8; 32] = hex::decode(&inp.input)
        .expect("valid hex input")
        .try_into()
        .expect("32 bytes");
    for _ in 0..inp.n_iters {
        let mut h = Sha256::new();
        h.update(cur);
        cur = h.finalize().into();
    }

    sp1_zkvm::io::commit(&PowOut {
        n_iters: inp.n_iters,
        input: inp.input,
        output: hex::encode(cur),
    });
}
