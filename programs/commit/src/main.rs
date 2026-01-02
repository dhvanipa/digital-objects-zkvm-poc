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
    1587596086, 298404220, 585479275, 474436278, 74356112, 727867099, 332757853, 582942523,
];
const STONE_VKEY_HASH: [u32; 8] = [
    1757596195, 1363166114, 741635403, 87936091, 524578768, 671105610, 620679257, 596851894,
];
const AXE_VKEY_HASH: [u32; 8] = [
    615685952, 1622331182, 784517521, 533537318, 560190469, 502351443, 390831125, 1477857554,
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
