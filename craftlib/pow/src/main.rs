//! A proof of work program

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use pow_program::PV;
use sha2::{Digest, Sha256};

const VKEY_HASH: [u32; 8] = [
    1421000253, 1824336850, 153211562, 1265532765, 1905838329, 757838639, 1641482252, 117455217,
];

pub fn main() {
    // 1) Decide base vs recursive step
    let has_prev = sp1_zkvm::io::read::<bool>();

    let prev: PV = if has_prev {
        let public_values = sp1_zkvm::io::read::<PV>();
        let public_values_digest = Sha256::digest(&bincode::serialize(&public_values).unwrap());

        sp1_zkvm::lib::verify::verify_sp1_proof(&VKEY_HASH, &public_values_digest.into());

        PV {
            count: public_values.count,
            base_input: public_values.base_input,
            x: public_values.x,
        }
    } else {
        // --- Base step ---
        let base_input = sp1_zkvm::io::read::<[u8; 32]>();
        PV {
            count: 0,
            base_input,
            x: base_input,
        }
    };

    // 2) Do one unit of work: hash once
    let x_next: [u8; 32] = Sha256::digest(&prev.x).into();
    let count_next = prev.count.wrapping_add(1);

    let commit_pv = PV {
        count: count_next,
        base_input: prev.base_input,
        x: x_next,
    };

    // 3) Commit the new PV (this becomes the "public values" for this proof)
    sp1_zkvm::io::commit(&commit_pv);
}
