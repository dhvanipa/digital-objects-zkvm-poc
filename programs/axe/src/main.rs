//! A simple program that crafts an object

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]

use axe_program::constants;
use common::{difficulty, ObjectInput, ObjectOutput};
use sha2::{Digest, Sha256};

sp1_zkvm::entrypoint!(main);

// TODO: find a way to auto-generate and share these constants
const WOOD_VKEY_HASH: [u32; 8] = [
    92693219, 536712614, 913175624, 1780277906, 1719680246, 1776819973, 1022706495, 931408120,
];
const STONE_VKEY_HASH: [u32; 8] = [
    44584754, 336036070, 1781962132, 1150135370, 439676485, 1927184313, 1849913332, 956193253,
];

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a system call which handles reading inputs
    // from the prover.
    let object_inp = sp1_zkvm::io::read::<ObjectInput>();

    assert!(object_inp.object.inputs.len() == 2, "Must have 2 inputs");
    assert!(
        object_inp.object.blueprint == constants::AXE_BLUEPRINT,
        "Blueprint must be axe"
    );

    let object_hash = object_inp.object.hash();
    assert!(
        difficulty(&object_hash) <= constants::AXE_MINING_MAX,
        "Object hash does not meet mining difficulty"
    );
    let empty_work = hex::encode([0u8; 32]);
    assert!(
        object_inp.work == empty_work,
        "Proof of work output must match object work"
    );

    // TODO: factor out common code for verifying objects
    let wood_input = sp1_zkvm::io::read::<ObjectOutput>();
    let wood_input_digest: [u8; 32] =
        Sha256::digest(&bincode::serialize(&wood_input).unwrap()).into();
    sp1_zkvm::lib::verify::verify_sp1_proof(&WOOD_VKEY_HASH, &wood_input_digest);
    // TODO: make this so the inputs can be in any order
    assert!(
        wood_input.hash == object_inp.object.inputs[0],
        "Missing wood input"
    );

    let stone_input = sp1_zkvm::io::read::<ObjectOutput>();
    let stone_input_digest: [u8; 32] =
        Sha256::digest(&bincode::serialize(&stone_input).unwrap()).into();
    sp1_zkvm::lib::verify::verify_sp1_proof(&STONE_VKEY_HASH, &stone_input_digest);
    assert!(
        stone_input.hash == object_inp.object.inputs[1],
        "Missing stone input"
    );

    // Write the output of the program.
    //
    // Behind the scenes, this also compiles down to a system call which handles writing
    // outputs to the prover.
    sp1_zkvm::io::commit(&ObjectOutput {
        hash: object_hash,
        consumed: object_inp.object.inputs,
    });
}
