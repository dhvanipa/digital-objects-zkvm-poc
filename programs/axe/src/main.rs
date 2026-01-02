//! A simple program that crafts an object

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]

use axe_program::constants;
use common::{difficulty, hex_to_vk_digest, ObjectInput, ObjectOutput};
use sha2::{Digest, Sha256};

sp1_zkvm::entrypoint!(main);

// TODO: find a way to auto-generate and share these constants, also store without having to decode
const WOOD_VKEY_HASH: &str = "1dc5f3be73dbe87875287d81592e666159cd2bb91b9a30a54e4c70c570523f45";
const STONE_VKEY_HASH: &str = "5d0865c4708a2df7685dee2f0c98e52d32674eee126a276c127276bd6e3d7ad2";

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
    sp1_zkvm::lib::verify::verify_sp1_proof(&hex_to_vk_digest(WOOD_VKEY_HASH), &wood_input_digest);
    // TODO: make this so the inputs can be in any order
    assert!(
        wood_input.hash == object_inp.object.inputs[0],
        "Missing wood input"
    );

    let stone_input = sp1_zkvm::io::read::<ObjectOutput>();
    let stone_input_digest: [u8; 32] =
        Sha256::digest(&bincode::serialize(&stone_input).unwrap()).into();
    sp1_zkvm::lib::verify::verify_sp1_proof(
        &hex_to_vk_digest(STONE_VKEY_HASH),
        &stone_input_digest,
    );
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
