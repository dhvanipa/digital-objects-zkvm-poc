//! A simple program that crafts an object

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]

use common::{difficulty, ObjectInput, ObjectOutput};
use sha2::{Digest, Sha256};

mod constants;

sp1_zkvm::entrypoint!(main);

// TODO: find a way to auto-generate and share these constants
const POW_VKEY_HASH: [u32; 8] = [
    1187928552, 1830467042, 1921204169, 530199403, 1883161345, 29298083, 1615915134, 1929779142,
];

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a system call which handles reading inputs
    // from the prover.
    let object_inp = sp1_zkvm::io::read::<ObjectInput>();

    assert!(object_inp.object.inputs.len() == 0, "Must have no inputs");
    assert!(
        object_inp.object.blueprint == constants::STONE_BLUEPRINT,
        "Blueprint must be stone"
    );

    let object_hash = object_inp.object.hash();
    assert!(
        difficulty(&object_hash) <= constants::STONE_MINING_MAX,
        "Object hash does not meet mining difficulty"
    );

    let pow_public_values = sp1_zkvm::io::read::<pow_program::PowOut>();
    let public_values_digest: [u8; 32] =
        Sha256::digest(&bincode::serialize(&pow_public_values).unwrap()).into();
    sp1_zkvm::lib::verify::verify_sp1_proof(&POW_VKEY_HASH, &public_values_digest);
    assert!(
        pow_public_values.n_iters == 3,
        "Proof of work must have 3 iterations"
    );
    assert!(
        pow_public_values.input == object_hash,
        "Proof of work input must match object hash"
    );
    assert!(
        pow_public_values.output == object_inp.work,
        "Proof of work output must match object work"
    );

    // Write the output of the program.
    //
    // Behind the scenes, this also compiles down to a system call which handles writing
    // outputs to the prover.
    sp1_zkvm::io::commit(&ObjectOutput {
        hash: object_hash,
        consumed: vec![],
    });
}
