//! A proof of work program

use risc0_zkvm::{guest::env, serde};

use axe_program::constants::AXE_BLUEPRINT;
use commit_program::{CommitIn, CommitOut};
use common::{ObjectHash, ObjectOutput};
use sha2::{Digest, Sha256};
use stone_program::constants::STONE_BLUEPRINT;
use wood_program::constants::WOOD_BLUEPRINT;

// use axe::AXE_PROGRAM_ID;
// use stone::STONE_PROGRAM_ID;
// use wood::WOOD_PROGRAM_ID;
// TODO: figure out why these imports aren't the latest ones
// use stone::STONE_PROGRAM_ID;
// use wood::WOOD_PROGRAM_ID;
const WOOD_PROGRAM_ID: [u32; 8] = [
    4051687538, 1391007321, 3957527642, 3411415860, 3924906641, 3128546228, 3897880847, 1969275840,
];
const STONE_PROGRAM_ID: [u32; 8] = [
    1209310465, 973615066, 2006126405, 3155197817, 658800578, 1149895044, 2927651235, 2778763347,
];
const AXE_PROGRAM_ID: [u32; 8] = [
    1030046717, 1750057497, 2835778131, 3049353812, 2668477960, 3919716371, 3158757716, 2741622414,
];

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
