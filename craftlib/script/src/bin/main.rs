use core::num;
use pow_program::PowIn;
use sha2::{Digest, Sha256};
use sp1_sdk::{
    include_elf, utils, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin,
};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// The ELF we want to execute inside the zkVM.
const POW_ELF: &[u8] = include_elf!("pow-program");
const STONE_ELF: &[u8] = include_elf!("stone-program");

fn main() {
    // Setup logging.
    utils::setup_logger();

    // The input stream that the program will read from using `sp1_zkvm::io::read`. Note that the
    // types of the elements in the input stream must match the types being read in the program.
    let mut stdin = SP1Stdin::new();

    // Create a `ProverClient` method.
    let client = ProverClient::from_env();

    // Setup for the step program
    let (pk_step, vk_step) = client.setup(POW_ELF);
    println!("Proving key hash (step): {:?}", vk_step.hash_u32());

    // Choose starting input
    let input: [u8; 32] = Sha256::digest(b"starting input").into();
    let num_work = 300u32;

    // --- Step 1 (base) ---
    stdin.write(&PowIn {
        n_iters: num_work,
        input,
    });

    let mut proof: SP1ProofWithPublicValues = client
        .prove(&pk_step, &stdin)
        .compressed()
        .run()
        .expect("proving failed");
    let pv = proof.public_values.read::<pow_program::PowOut>();

    client
        .verify(&proof, &vk_step)
        .expect("verification failed");

    println!("final pv = {:?}", pv);

    // // // Test a round trip of proof serialization and deserialization.
    // proof
    //     .save("proof-with-pis.bin")
    //     .expect("saving proof failed");
    // save_proof_as_json(&proof, "proof-with-pis.json").expect("saving proof as json failed");

    // // let mut deserialized_proof =
    // //     SP1ProofWithPublicValues::load("proof-with-pis.bin").expect("loading proof failed");
    // let mut deserialized_proof =
    //     load_proof_from_json("proof-with-pis.json").expect("loading proof from json failed");

    // let n = deserialized_proof.public_values.read::<u32>();
    // let a = deserialized_proof.public_values.read::<u32>();
    // let b = deserialized_proof.public_values.read::<u32>();
    // println!("n (from deserialized proof): {}", n);
    // println!("a (from deserialized proof): {}", a);
    // println!("b (from deserialized proof): {}", b);

    // // Verify the deserialized proof.
    // client
    //     .verify(&deserialized_proof, &vk)
    //     .expect("verification failed");

    // println!("successfully generated and verified proof for the program!")
}
