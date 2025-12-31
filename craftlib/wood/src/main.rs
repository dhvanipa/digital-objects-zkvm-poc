//! A simple program that crafts an object

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]

use common::{top_u64_be, ObjectInput, ObjectOutput};
sp1_zkvm::entrypoint!(main);

mod constants;

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a system call which handles reading inputs
    // from the prover.
    let object_inp = sp1_zkvm::io::read::<ObjectInput>();

    assert!(object_inp.object.inputs.len() == 0, "Must have no inputs");
    assert!(
        object_inp.object.blueprint == constants::WOOD_BLUEPRINT,
        "Blueprint must be wood"
    );

    let object_hash: [u8; 32] = object_inp.object.hash();
    assert!(
        top_u64_be(object_hash) <= constants::WOOD_MINING_MAX,
        "Object hash does not meet mining difficulty"
    );
    let empty_work: [u8; 32] = [0u8; 32];
    assert!(
        object_inp.work == empty_work,
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
