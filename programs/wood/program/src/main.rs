//! A simple program that crafts an object

use risc0_zkvm::guest::env;

use common::{difficulty, ObjectInput, ObjectOutput};

mod constants;

fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a system call which handles reading inputs
    // from the prover.
    let object_inp = env::read::<ObjectInput>();

    assert!(object_inp.object.inputs.len() == 0, "Must have no inputs");
    assert!(
        object_inp.object.blueprint == constants::WOOD_BLUEPRINT,
        "Blueprint must be wood"
    );

    let object_hash = object_inp.object.hash();
    assert!(
        difficulty(&object_hash) <= constants::WOOD_MINING_MAX,
        "Object hash does not meet mining difficulty"
    );
    let empty_work = hex::encode([0u8; 32]);
    assert!(
        object_inp.work == empty_work,
        "Proof of work output must match object work"
    );

    // Write the output of the program.
    //
    // Behind the scenes, this also compiles down to a system call which handles writing
    // outputs to the prover.
    env::commit(&ObjectOutput {
        hash: object_hash,
        consumed: vec![],
    });
}
