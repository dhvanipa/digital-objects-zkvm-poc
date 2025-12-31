use std::vec;

use sp1_sdk::{
    include_elf, utils, EnvProver, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues,
    SP1Stdin,
};

use axe_program::constants::{AXE_BLUEPRINT, AXE_MINING_MAX};
use common::{object_hash_excluding_work, top_u64_be, Object, ObjectInput, ObjectOutput};
use pow_program::{PowIn, PowOut};
use stone_program::constants::{STONE_BLUEPRINT, STONE_MINING_MAX};
use wood_program::constants::{WOOD_BLUEPRINT, WOOD_MINING_MAX};

mod save;

const POW_ELF: &[u8] = include_elf!("pow-program");
const STONE_ELF: &[u8] = include_elf!("stone-program");
const WOOD_ELF: &[u8] = include_elf!("wood-program");
const AXE_ELF: &[u8] = include_elf!("axe-program");

fn mine_object(blueprint: &str, max_difficulty: u64, inputs: Vec<[u8; 32]>) -> (Object, [u8; 32]) {
    let key = {
        let bytes: [u8; 32] = rand::random();
        hex::encode(bytes)
    };

    for seed in 0u32..=u32::MAX {
        let obj = Object {
            key: key.clone(),
            inputs: inputs.clone(),
            seed,
            blueprint: blueprint.to_string(),
            work: [0u8; 32],
        };

        let h = object_hash_excluding_work(&obj);
        if top_u64_be(h) <= max_difficulty {
            return (obj, h);
        }
    }

    panic!("failed to mine {} object", blueprint);
}

fn create_pow_proof(
    client: &EnvProver,
    pow_pk: &sp1_sdk::SP1ProvingKey,
    pow_vk: &sp1_sdk::SP1VerifyingKey,
    n_iters: u32,
    input: [u8; 32],
) -> (PowOut, SP1Proof) {
    let mut pow_stdin = SP1Stdin::new();
    pow_stdin.write(&PowIn { n_iters, input });

    let mut pow_proof: SP1ProofWithPublicValues = client
        .prove(pow_pk, &pow_stdin)
        .compressed()
        .run()
        .expect("pow proving failed");

    client
        .verify(&pow_proof, pow_vk)
        .expect("pow verify failed");

    let pow_out: PowOut = pow_proof.public_values.read();

    let SP1Proof::Compressed(compressed_proof) = pow_proof.proof else {
        panic!("expected compressed proof")
    };

    (pow_out, SP1Proof::Compressed(compressed_proof))
}

fn create_stone_object(
    client: &EnvProver,
    pow_pk: &sp1_sdk::SP1ProvingKey,
    pow_vk: &sp1_sdk::SP1VerifyingKey,
    stone_pk: &sp1_sdk::SP1ProvingKey,
    stone_vk: &sp1_sdk::SP1VerifyingKey,
) -> (Object, SP1ProofWithPublicValues) {
    let (mut obj, obj_hash) = mine_object(STONE_BLUEPRINT, STONE_MINING_MAX, vec![]);
    println!("Mined stone: seed={}, hash={:x?}", obj.seed, obj_hash);

    let (pow_out, pow_proof) = create_pow_proof(client, pow_pk, pow_vk, 3, obj_hash);
    obj.work = pow_out.output;

    let mut stone_stdin = SP1Stdin::new();
    stone_stdin.write(&ObjectInput {
        hash: obj_hash,
        object: obj.clone(),
    });
    stone_stdin.write(&pow_out);

    let SP1Proof::Compressed(compressed_proof) = pow_proof else {
        panic!("expected compressed proof")
    };
    stone_stdin.write_proof(*compressed_proof, pow_vk.clone().vk);

    let mut stone_proof: SP1ProofWithPublicValues = client
        .prove(stone_pk, &stone_stdin)
        .compressed()
        .run()
        .expect("stone proving failed");

    client
        .verify(&stone_proof, stone_vk)
        .expect("stone verify failed");

    let committed_hash: [u8; 32] = stone_proof.public_values.read();
    println!("Stone committed hash: {:x?}", committed_hash);

    (obj, stone_proof)
}

fn create_wood_object(
    client: &EnvProver,
    wood_pk: &sp1_sdk::SP1ProvingKey,
    wood_vk: &sp1_sdk::SP1VerifyingKey,
) -> (Object, SP1ProofWithPublicValues) {
    let (obj, obj_hash) = mine_object(WOOD_BLUEPRINT, WOOD_MINING_MAX, vec![]);
    println!("Mined wood: seed={}, hash={:x?}", obj.seed, obj_hash);

    let mut wood_stdin = SP1Stdin::new();
    wood_stdin.write(&ObjectInput {
        hash: obj_hash,
        object: obj.clone(),
    });

    let mut wood_proof: SP1ProofWithPublicValues = client
        .prove(wood_pk, &wood_stdin)
        .compressed()
        .run()
        .expect("wood proving failed");

    client
        .verify(&wood_proof, wood_vk)
        .expect("wood verify failed");

    let committed_hash: [u8; 32] = wood_proof.public_values.read();
    println!("Wood committed hash: {:x?}", committed_hash);

    (obj, wood_proof)
}

fn create_axe_object(
    client: &EnvProver,
    axe_pk: &sp1_sdk::SP1ProvingKey,
    axe_vk: &sp1_sdk::SP1VerifyingKey,
    stone_vk: &sp1_sdk::SP1VerifyingKey,
    wood_vk: &sp1_sdk::SP1VerifyingKey,
    wood_hash: [u8; 32],
    wood_proof: SP1ProofWithPublicValues,
    stone_hash: [u8; 32],
    stone_proof: SP1ProofWithPublicValues,
) -> (Object, SP1ProofWithPublicValues) {
    let (obj, obj_hash) = mine_object(AXE_BLUEPRINT, AXE_MINING_MAX, vec![wood_hash, stone_hash]);
    println!("Created axe: seed={}, hash={:x?}", obj.seed, obj_hash);

    let mut axe_stdin = SP1Stdin::new();
    axe_stdin.write(&ObjectInput {
        hash: obj_hash,
        object: obj.clone(),
    });

    let wood_output = ObjectOutput {
        hash: wood_hash,
        consumed: vec![],
    };
    axe_stdin.write(&wood_output);

    let SP1Proof::Compressed(wood_compressed) = wood_proof.proof else {
        panic!("expected compressed proof")
    };
    axe_stdin.write_proof(*wood_compressed, wood_vk.clone().vk);

    let stone_output = ObjectOutput {
        hash: stone_hash,
        consumed: vec![],
    };
    axe_stdin.write(&stone_output);

    let SP1Proof::Compressed(stone_compressed) = stone_proof.proof else {
        panic!("expected compressed proof")
    };
    axe_stdin.write_proof(*stone_compressed, stone_vk.clone().vk);

    let mut axe_proof: SP1ProofWithPublicValues = client
        .prove(axe_pk, &axe_stdin)
        .compressed()
        .run()
        .expect("axe proving failed");

    client
        .verify(&axe_proof, axe_vk)
        .expect("axe verify failed");

    let committed_output: ObjectOutput = axe_proof.public_values.read();
    println!("Axe committed hash: {:x?}", committed_output.hash);

    (obj, axe_proof)
}

fn main() {
    utils::setup_logger();

    let client = ProverClient::from_env();

    println!("Setting up proving/verifying keys...");
    let (pow_pk, pow_vk) = client.setup(POW_ELF);
    let (stone_pk, stone_vk) = client.setup(STONE_ELF);
    let (wood_pk, wood_vk) = client.setup(WOOD_ELF);
    let (axe_pk, axe_vk) = client.setup(AXE_ELF);
    println!("pow vk {:?}", wood_vk.hash_u32());
    println!("wood vk {:?}", wood_vk.hash_u32());
    println!("stone vk {:?}", stone_vk.hash_u32());
    println!("axe vk {:?}", axe_vk.hash_u32());

    std::fs::create_dir_all("objects").expect("failed to create objects directory");

    let num_stones = 1;
    let num_woods = 1;
    let num_axes = 1;

    let mut stone_objects = Vec::new();
    let mut wood_objects = Vec::new();

    for i in 1..=num_stones {
        println!("\n=== Creating Stone {} ===", i);
        let (obj, proof) = create_stone_object(&client, &pow_pk, &pow_vk, &stone_pk, &stone_vk);
        let obj_hash = object_hash_excluding_work(&obj);
        let filename = format!("objects/stone_{}.json", i);
        save::save_object_as_json(&obj, &proof, &filename).expect("failed to save stone");
        println!("Saved to {}", filename);
        stone_objects.push((obj_hash, proof));
    }

    for i in 1..=num_woods {
        println!("\n=== Creating Wood {} ===", i);
        let (obj, proof) = create_wood_object(&client, &wood_pk, &wood_vk);
        let obj_hash = object_hash_excluding_work(&obj);
        let filename = format!("objects/wood_{}.json", i);
        save::save_object_as_json(&obj, &proof, &filename).expect("failed to save wood");
        println!("Saved to {}", filename);
        wood_objects.push((obj_hash, proof));
    }

    for i in 1..=num_axes {
        println!("\n=== Creating Axe {} ===", i);
        let (wood_hash, wood_proof) = wood_objects.pop().expect("need wood for axe");
        let (stone_hash, stone_proof) = stone_objects.pop().expect("need stone for axe");

        let (obj, proof) = create_axe_object(
            &client,
            &axe_pk,
            &axe_vk,
            &stone_vk,
            &wood_vk,
            wood_hash,
            wood_proof,
            stone_hash,
            stone_proof,
        );
        let filename = format!("objects/axe_{}.json", i);
        save::save_object_as_json(&obj, &proof, &filename).expect("failed to save axe");
        println!("Saved to {}", filename);
    }

    println!("\n✓ All objects created successfully!");
}
