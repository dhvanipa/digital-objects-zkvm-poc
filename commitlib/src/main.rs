use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, Receipt};

use ::utils::{save_proof_as_json, ObjectJson};
use commit_program::{CommitIn, CommitOut, ObjectOutputWithType};
use common::ObjectOutput;
use sha2::{Digest, Sha256};

use crate::eth::send_blob_tx;

use commit::{COMMIT_PROGRAM_ELF, COMMIT_PROGRAM_ID};

mod eth;

fn commit_objects(
    prover: &impl risc0_zkvm::Prover,
    prover_opts: &ProverOpts,
    object_jsons: Vec<ObjectJson>,
) -> (CommitOut, Receipt) {
    let mut objects: Vec<ObjectOutputWithType> = Vec::new();
    for obj_json in &object_jsons {
        let object_output: ObjectOutput = obj_json.proof.journal.decode().unwrap();

        objects.push(ObjectOutputWithType {
            hash: object_output.hash.clone(),
            consumed: object_output.consumed.clone(),
            blueprint: obj_json.object.blueprint.clone(),
        });
    }

    let commit_input = CommitIn { objects: objects };
    let mut env_builder = ExecutorEnv::builder();
    let env_builder = env_builder.write(&commit_input).unwrap();

    let env_builder = object_jsons
        .into_iter()
        .fold(env_builder, |builder, obj_json| {
            let object_output: ObjectOutput = obj_json.proof.journal.decode().unwrap();
            builder
                .add_assumption(obj_json.proof.clone())
                .write(&object_output)
                .unwrap()
        });

    let env = env_builder.build().unwrap();

    let start = std::time::Instant::now();
    let commit_proof = prover
        .prove_with_opts(env, COMMIT_PROGRAM_ELF, prover_opts)
        .unwrap();
    let duration = start.elapsed();
    println!("\nTotal commit proof creation time: {:?}", duration);

    commit_proof.receipt.verify(COMMIT_PROGRAM_ID).unwrap();

    let committed_output: CommitOut = commit_proof.receipt.journal.decode().unwrap();

    (committed_output, commit_proof.receipt)
}

#[tokio::main]
async fn main() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    // Obtain the default prover.
    let prover = default_prover();
    let prover_opts = ProverOpts::groth16();

    std::fs::create_dir_all("commitments").expect("failed to create commitments directory");

    println!("commit program id {:?}", COMMIT_PROGRAM_ID);

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <objects_folder>", args[0]);
        std::process::exit(1);
    }

    let objects_folder = &args[1];
    let mut objects: Vec<ObjectJson> = Vec::new();

    let entries = std::fs::read_dir(objects_folder).unwrap_or_else(|e| {
        eprintln!("Failed to read directory {}: {}", objects_folder, e);
        std::process::exit(1);
    });

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            match ObjectJson::from_json_file(path.to_str().unwrap()) {
                Ok(obj_json) => {
                    println!("Loaded object from {}", path.display());
                    objects.push(obj_json);
                }
                Err(e) => {
                    eprintln!("Failed to load {}: {}", path.display(), e);
                    std::process::exit(1);
                }
            }
        }
    }

    if objects.is_empty() {
        eprintln!("No JSON files found in {}", objects_folder);
        std::process::exit(1);
    }

    let (committed_output, commit_proof) = commit_objects(&prover, &prover_opts, objects);
    println!("Committed output: {:?}", committed_output);

    let commit_proof_hash: [u8; 32] = Sha256::digest(
        &bincode::serialize(&commit_proof).expect("Failed to serialize commit proof"),
    )
    .into();
    println!("Commit proof hash: {}", hex::encode(commit_proof_hash));

    save_proof_as_json(
        &commit_proof,
        &format!("commitments/{}.json", hex::encode(commit_proof_hash)),
    )
    .expect("failed to save commit proof");

    // Note: We cannot send the full commit proof as blob data due to size limits.
    // let commitment_blob_data: Vec<u8> =
    //     bincode::serialize(&commit_proof).expect("failed to serialize commit proof");

    send_blob_tx(&commit_proof_hash)
        .await
        .expect("failed to send blob transaction");

    println!("\n✓ All objects committed successfully!");
}
