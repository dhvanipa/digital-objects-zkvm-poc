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
    687550195, 1793166740, 338494431, 1946809861, 1472814873, 1435689528, 136791663, 372439300,
];
const STONE_VKEY_HASH: [u32; 8] = [
    526909274, 1664124846, 690861976, 1172288529, 741496193, 1239481263, 419179509, 1174901531,
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

    let object_hash: [u8; 32] = object_inp.object.hash();
    assert!(
        difficulty(object_hash) <= constants::AXE_MINING_MAX,
        "Object hash does not meet mining difficulty"
    );
    let empty_work: [u8; 32] = [0u8; 32];
    assert!(
        object_inp.work == empty_work,
        "Proof of work output must match object work"
    );

    // TODO: factor out common code for verifying objects
    let wood_input = sp1_zkvm::io::read::<ObjectOutput>();
    let wood_input_digest = Sha256::digest(&bincode::serialize(&wood_input).unwrap());
    sp1_zkvm::lib::verify::verify_sp1_proof(&WOOD_VKEY_HASH, &wood_input_digest.into());
    // TODO: make this so the inputs can be in any order
    assert!(
        wood_input.hash == object_inp.object.inputs[0],
        "Missing wood input"
    );

    let stone_input = sp1_zkvm::io::read::<ObjectOutput>();
    let stone_input_digest = Sha256::digest(&bincode::serialize(&stone_input).unwrap());
    sp1_zkvm::lib::verify::verify_sp1_proof(&STONE_VKEY_HASH, &stone_input_digest.into());
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
