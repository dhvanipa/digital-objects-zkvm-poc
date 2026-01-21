//! A proof of work program

use risc0_zkvm::{guest::env, serde};

use axe_program::constants::AXE_BLUEPRINT;
use commit_program::{CommitIn, CommitOut};
use common::{ObjectHash, ObjectOutput};
use sha2::{Digest, Sha256};
use stone_program::constants::STONE_BLUEPRINT;
use wood_program::constants::WOOD_BLUEPRINT;

use axe::AXE_PROGRAM_ID;
use stone::STONE_PROGRAM_ID;
use wood::WOOD_PROGRAM_ID;

fn main() {
    let inp = env::read::<CommitIn>();

    let mut created: Vec<ObjectHash> = Vec::new();
    let mut consumed: Vec<ObjectHash> = Vec::new();

    for object in inp.objects.iter() {
        let object_output = ObjectOutput {
            hash: object.hash.clone(),
            consumed: object.consumed.clone(),
        };
        // Verify proof
        match object.blueprint.as_str() {
            WOOD_BLUEPRINT => {
                env::verify(WOOD_PROGRAM_ID, &serde::to_vec(&object_output).unwrap()).unwrap();
            }
            STONE_BLUEPRINT => {
                env::verify(STONE_PROGRAM_ID, &serde::to_vec(&object_output).unwrap()).unwrap();
            }
            AXE_BLUEPRINT => {
                env::verify(AXE_PROGRAM_ID, &serde::to_vec(&object_output).unwrap()).unwrap();
            }
            _ => panic!("unknown blueprint"),
        }
        created.push(object.hash.clone());
        for inp in object.consumed.iter() {
            consumed.push(inp.clone());
        }
    }

    env::commit(&CommitOut { created, consumed });
}
