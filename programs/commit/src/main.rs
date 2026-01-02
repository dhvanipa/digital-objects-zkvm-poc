//! A proof of work program

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use axe_program::constants::AXE_BLUEPRINT;
use commit_program::{CommitIn, CommitOut};
use common::{hex_to_vk_digest, ObjectHash, ObjectOutput};
use sha2::{Digest, Sha256};
use stone_program::constants::STONE_BLUEPRINT;
use wood_program::constants::WOOD_BLUEPRINT;

// TODO: find a way to auto-generate and share these constants, also store without having to decode
const WOOD_VKEY_HASH: &str = "1dc5f3be73dbe87875287d81592e666159cd2bb91b9a30a54e4c70c570523f45";
const STONE_VKEY_HASH: &str = "5d0865c4708a2df7685dee2f0c98e52d32674eee126a276c127276bd6e3d7ad2";
const AXE_VKEY_HASH: &str = "2656118a5e39427a0943d08447d372cf1c6d276b396f90f31cbc87706db3c6ca";

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
                sp1_zkvm::lib::verify::verify_sp1_proof(
                    &hex_to_vk_digest(WOOD_VKEY_HASH),
                    &object_output_digest,
                );
            }
            STONE_BLUEPRINT => {
                sp1_zkvm::lib::verify::verify_sp1_proof(
                    &hex_to_vk_digest(STONE_VKEY_HASH),
                    &object_output_digest,
                );
            }
            AXE_BLUEPRINT => {
                sp1_zkvm::lib::verify::verify_sp1_proof(
                    &hex_to_vk_digest(AXE_VKEY_HASH),
                    &object_output_digest,
                );
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
