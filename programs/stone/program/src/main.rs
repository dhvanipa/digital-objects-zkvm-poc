//! A simple program that crafts an object

use risc0_zkvm::{guest::env, serde};

use common::{difficulty, ObjectInput, ObjectOutput};

mod constants;

// TODO: figure out why these imports aren't the latest ones
// use pow::POW_PROGRAM_ID;
const POW_PROGRAM_ID: [u32; 8] = [
    1649781456, 1167951597, 353527003, 4019636127, 1570962558, 3626274108, 367129753, 1095461449,
];

fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a system call which handles reading inputs
    // from the prover.
    let object_inp = env::read::<ObjectInput>();

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

    let pow_public_values = env::read::<pow_program::PowOut>();
    env::verify(POW_PROGRAM_ID, &serde::to_vec(&pow_public_values).unwrap()).unwrap();
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
    env::commit(&ObjectOutput {
        hash: object_hash,
        consumed: vec![],
    });
}
