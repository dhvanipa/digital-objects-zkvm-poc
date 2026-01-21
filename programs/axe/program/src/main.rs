//! A simple program that crafts an object

use risc0_zkvm::{guest::env, serde};

use axe_program::constants;
use common::{difficulty, ObjectInput, ObjectOutput};

// TODO: figure out why these imports aren't the latest ones
// use stone::STONE_PROGRAM_ID;
// use wood::WOOD_PROGRAM_ID;
const WOOD_PROGRAM_ID: [u32; 8] = [
    4051687538, 1391007321, 3957527642, 3411415860, 3924906641, 3128546228, 3897880847, 1969275840,
];
const STONE_PROGRAM_ID: [u32; 8] = [
    1209310465, 973615066, 2006126405, 3155197817, 658800578, 1149895044, 2927651235, 2778763347,
];

fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a system call which handles reading inputs
    // from the prover.
    let object_inp = env::read::<ObjectInput>();

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
    let wood_input = env::read::<ObjectOutput>();
    env::verify(WOOD_PROGRAM_ID, &serde::to_vec(&wood_input).unwrap()).unwrap();
    // TODO: make this so the inputs can be in any order
    assert!(
        wood_input.hash == object_inp.object.inputs[0],
        "Missing wood input"
    );

    let stone_input = env::read::<ObjectOutput>();
    env::verify(STONE_PROGRAM_ID, &serde::to_vec(&stone_input).unwrap()).unwrap();
    assert!(
        stone_input.hash == object_inp.object.inputs[1],
        "Missing stone input"
    );

    // Write the output of the program.
    //
    // Behind the scenes, this also compiles down to a system call which handles writing
    // outputs to the prover.
    env::commit(&ObjectOutput {
        hash: object_hash,
        consumed: object_inp.object.inputs,
    });
}
