//! A proof of work program

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use pow_program::{PowIn, PowOut};
use sha2::{Digest, Sha256};

pub fn main() {
    let inp: PowIn = sp1_zkvm::io::read::<PowIn>();

    let mut cur = inp.input;
    for _ in 0..inp.n_iters {
        let mut h = Sha256::new();
        h.update(cur);
        cur = h.finalize().into();
    }

    sp1_zkvm::io::commit(&PowOut {
        n_iters: inp.n_iters,
        input: inp.input,
        output: cur,
    });
}
