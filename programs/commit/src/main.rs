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
    109196305, 262471982, 483527702, 357546556, 1131681982, 211602804, 1093354595, 1784903095,
];
const STONE_VKEY_HASH: [u32; 8] = [
    798238549, 1320429416, 1994312076, 723619578, 1320567298, 1561260577, 1612222587, 890584611,
];
const AXE_VKEY_HASH: [u32; 8] = [
    127085715, 319382426, 2006292001, 974531127, 1273546570, 224535709, 1425381621, 1084853484,
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
