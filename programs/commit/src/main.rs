//! A proof of work program

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use axe_program::constants::AXE_BLUEPRINT;
use commit_program::{CommitIn, CommitOut};
use common::{ObjectHash, ObjectOutput};
use sha2::{Digest, Sha256};
use stone_program::constants::STONE_BLUEPRINT;
use wood_program::constants::WOOD_BLUEPRINT;

// TODO: find a way to auto-generate and share these constants
const WOOD_VKEY_HASH: [u32; 8] = [
    92693219, 536712614, 913175624, 1780277906, 1719680246, 1776819973, 1022706495, 931408120,
];
const STONE_VKEY_HASH: [u32; 8] = [
    44584754, 336036070, 1781962132, 1150135370, 439676485, 1927184313, 1849913332, 956193253,
];
const AXE_VKEY_HASH: [u32; 8] = [
    44584754, 336036070, 1781962132, 1150135370, 439676485, 1927184313, 1849913332, 956193253,
];

pub fn main() {
    let inp = sp1_zkvm::io::read::<CommitIn>();

    let mut created: Vec<ObjectHash> = Vec::new();
    let mut consumed: Vec<ObjectHash> = Vec::new();

    for object in inp.objects.iter() {
        let object_output = ObjectOutput {
            hash: object.hash.clone(),
            consumed: object.consumed.clone(),
        };
        let object_output_digest: [u8; 32] =
            Sha256::digest(&bincode::serialize(&object_output).unwrap()).into();
        // Verify proof
        match object.blueprint.as_str() {
            WOOD_BLUEPRINT => {
                sp1_zkvm::lib::verify::verify_sp1_proof(&WOOD_VKEY_HASH, &object_output_digest);
            }
            STONE_BLUEPRINT => {
                sp1_zkvm::lib::verify::verify_sp1_proof(&STONE_VKEY_HASH, &object_output_digest);
            }
            AXE_BLUEPRINT => {
                sp1_zkvm::lib::verify::verify_sp1_proof(&AXE_VKEY_HASH, &object_output_digest);
            }
            _ => panic!("unknown blueprint"),
        }
        created.push(object.hash.clone());
        for inp in object.consumed.iter() {
            consumed.push(inp.clone());
        }
    }

    sp1_zkvm::io::commit(&CommitOut { created, consumed });
}
