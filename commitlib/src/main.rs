use std::vec;

use ::utils::ObjectJson;
use commit_program::{CommitIn, CommitOut, ObjectOutputWithType};
use common::ObjectOutput;
use sp1_sdk::{
    include_elf, utils, EnvProver, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues,
    SP1Stdin,
};

const COMMIT_ELF: &[u8] = include_elf!("commit-program");

fn commit_objects(
    client: &EnvProver,
    object_jsons: Vec<ObjectJson>,
    commit_pk: &sp1_sdk::SP1ProvingKey,
    commit_vk: &sp1_sdk::SP1VerifyingKey,
) -> (CommitOut, SP1ProofWithPublicValues) {
    let mut commit_stdin = SP1Stdin::new();

    let mut objects: Vec<ObjectOutputWithType> = Vec::new();
    for obj_json in &object_jsons {
        let object_output: ObjectOutput = obj_json.proof.public_values.clone().read();

        objects.push(ObjectOutputWithType {
            hash: object_output.hash.clone(),
            consumed: object_output.consumed.clone(),
            blueprint: obj_json.object.blueprint.clone(),
        });
    }

    let commit_input = CommitIn { objects: objects };
    commit_stdin.write(&commit_input);

    for obj_json in object_jsons {
        let object_output: ObjectOutput = obj_json.proof.public_values.clone().read();
        commit_stdin.write(&object_output);

        let SP1Proof::Compressed(obj_compressed) = obj_json.proof.proof else {
            panic!("expected compressed proof")
        };
        let vk = obj_compressed.vk.clone();
        commit_stdin.write_proof(*obj_compressed, vk);
    }

    let mut commit_proof: SP1ProofWithPublicValues = client
        .prove(commit_pk, &commit_stdin)
        .compressed()
        .run()
        .expect("commit proving failed");

    client
        .verify(&commit_proof, commit_vk)
        .expect("commit verify failed");

    let committed_output: CommitOut = commit_proof.public_values.read();

    (committed_output, commit_proof)
}

fn main() {
    utils::setup_logger();

    let client = ProverClient::from_env();

    println!("Setting up proving/verifying keys...");
    let (commit_pk, commit_vk) = client.setup(COMMIT_ELF);
    println!("commit program vk {:?}", commit_vk.hash_u32());

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <object1.json> [object2.json] ...", args[0]);
        std::process::exit(1);
    }

    let mut objects: Vec<ObjectJson> = Vec::new();
    for path in &args[1..] {
        match ObjectJson::from_json_file(path) {
            Ok(obj_json) => {
                println!("Loaded object from {}", path);
                objects.push(obj_json);
            }
            Err(e) => {
                eprintln!("Failed to load {}: {}", path, e);
                std::process::exit(1);
            }
        }
    }

    let (committed_output, _commit_proof) =
        commit_objects(&client, objects, &commit_pk, &commit_vk);
    println!("Committed output: {:?}", committed_output);

    println!("\n✓ All objects committed successfully!");
}
