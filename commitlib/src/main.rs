use std::vec;

use sp1_sdk::{
    include_elf, utils, EnvProver, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues,
    SP1Stdin,
};

const COMMIT_ELF: &[u8] = include_elf!("commit-program");

// fn commit_objects(client: &EnvProver, object_jsons: Vec<ObjectJson>) {}

fn main() {
    utils::setup_logger();

    let client = ProverClient::from_env();

    println!("Setting up proving/verifying keys...");
    let (commit_pk, commit_vk) = client.setup(COMMIT_ELF);
    println!("commit program vk {:?}", commit_vk.hash_u32());

    std::fs::create_dir_all("objects").expect("failed to create objects directory");

    println!("\n✓ All objects committed successfully!");
}
