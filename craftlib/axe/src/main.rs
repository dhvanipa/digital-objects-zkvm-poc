//! A simple program that crafts an object

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]

use axe_program::constants;
use common::{object_hash_excluding_work, top_u64_be, ObjectInput, ObjectOutput};
use sha2::{Digest, Sha256};

sp1_zkvm::entrypoint!(main);

const WOOD_VKEY_HASH: [u32; 8] = [
    967837498, 927415804, 215094840, 937742378, 334789233, 318139022, 1730081193, 260594501,
];
const STONE_VKEY_HASH: [u32; 8] = [
    904330211, 848037444, 1914903530, 1760434633, 822770919, 207072988, 1877340267, 293418241,
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

    let object_hash: [u8; 32] = object_hash_excluding_work(&object_inp.object);
    assert!(
        object_hash == object_inp.hash,
        "Object hash does not match expected hash"
    );
    assert!(
        top_u64_be(object_hash) <= constants::AXE_MINING_MAX,
        "Object hash does not meet mining difficulty"
    );
    let empty_work: [u8; 32] = [0u8; 32];
    assert!(
        object_inp.object.work == empty_work,
        "Proof of work output must match object work"
    );

    let wood_input = sp1_zkvm::io::read::<ObjectOutput>();
    let wood_input_digest = Sha256::digest(&bincode::serialize(&wood_input).unwrap());
    sp1_zkvm::lib::verify::verify_sp1_proof(&WOOD_VKEY_HASH, &wood_input_digest.into());
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
        hash: object_inp.hash,
        consumed: object_inp.object.inputs,
    });
}
