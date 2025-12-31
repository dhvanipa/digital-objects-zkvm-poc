use sha2::{Digest, Sha256};
use sp1_sdk::{
    include_elf, utils, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin,
};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_elf!("fibonacci-program");
const POW_ELF: &[u8] = include_elf!("pow-program");

fn save_proof_as_json(
    proof: &SP1ProofWithPublicValues,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(proof)?;
    let mut file = File::create(path.as_ref())?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn load_proof_from_json(
    path: impl AsRef<Path>,
) -> Result<SP1ProofWithPublicValues, Box<dyn std::error::Error>> {
    let file = File::open(path.as_ref())?;
    let proof = serde_json::from_reader(file)?;
    Ok(proof)
}

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

    // Choose starting input
    let base_input: [u8; 32] = Sha256::digest(b"starting input").into();

    // --- Step 1 (base) ---
    stdin.write(&false); // has_prev = false
    stdin.write(&base_input); // base_input

    let mut proof: SP1ProofWithPublicValues = client
        .prove(&pk_step, &stdin)
        .compressed()
        .run()
        .expect("proving failed");
    let pv = proof.public_values.read::<pow_program::PV>();
    println!("Step 1 proof generated with count = {:?}", pv);

    client
        .verify(&proof, &vk_step)
        .expect("verification failed");

    // --- Steps 2..N ---
    let n_steps = 10u32;
    let mut prev_proof = proof;
    let mut prev_pv = pv;

    for _ in 2..=n_steps {
        let mut stdin = SP1Stdin::new();
        stdin.write(&true); // has_prev = true
        stdin.write(&prev_pv);
        let SP1Proof::Compressed(proof) = prev_proof.proof else {
            panic!()
        };
        stdin.write_proof(*proof, vk_step.clone().vk);

        let mut next_proof = client
            .prove(&pk_step, &stdin)
            .compressed()
            .run()
            .expect("proving failed");
        let next_pv = next_proof.public_values.read::<pow_program::PV>();

        client
            .verify(&next_proof, &vk_step)
            .expect("verification failed");

        prev_proof = next_proof;
        prev_pv = next_pv;
    }

    println!("final pv = {:?}", prev_pv);

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
