//! A simple program that crafts a stone

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]

use sha2::{Digest, Sha256};
use stone_program::ObjectInput;
sp1_zkvm::entrypoint!(main);

const POW_VKEY_HASH: [u32; 8] = [
    400740169, 1082219568, 1402990414, 603044658, 1254365470, 1793183360, 321136646, 830585851,
];

const STONE_MINING_MAX: u64 = 0x0020_0000_0000_0000;

fn top_u64_be(hash: [u8; 32]) -> u64 {
    u64::from_be_bytes(hash[0..8].try_into().unwrap())
}

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a system call which handles reading inputs
    // from the prover.
    let object_inp = sp1_zkvm::io::read::<ObjectInput>();

    assert!(object_inp.object.inputs.len() == 0, "Must have no inputs");
    assert!(
        object_inp.object.blueprint == "stone",
        "Blueprint must be stone"
    );

    let object_hash: [u8; 32] = {
        // Use a deterministic serialization. bincode is deterministic *given the same config*.
        let bytes = bincode::serialize(&object_inp.object).expect("serialize Object");
        let digest = Sha256::digest(&bytes);
        digest.into()
    };
    assert!(
        object_hash == object_inp.hash,
        "Object hash does not match expected hash"
    );
    assert!(
        top_u64_be(object_hash) <= STONE_MINING_MAX,
        "Object hash does not meet mining difficulty"
    );

    let pow_public_values = sp1_zkvm::io::read::<pow_program::PowOut>();
    let public_values_digest = Sha256::digest(&bincode::serialize(&pow_public_values).unwrap());
    sp1_zkvm::lib::verify::verify_sp1_proof(&POW_VKEY_HASH, &public_values_digest.into());
    assert!(
        pow_public_values.n_iters == 3,
        "Proof of work must have 3 iterations"
    );
    assert!(
        pow_public_values.input == object_hash,
        "Proof of work input must match object hash"
    );
    assert!(
        pow_public_values.output == object_inp.object.work,
        "Proof of work output must match object work"
    );

    // Write the output of the program.
    //
    // Behind the scenes, this also compiles down to a system call which handles writing
    // outputs to the prover.
    sp1_zkvm::io::commit(&object_hash);
}
