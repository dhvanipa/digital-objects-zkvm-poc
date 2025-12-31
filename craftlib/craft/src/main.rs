use sp1_sdk::{
    include_elf, utils, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin,
};

use common::{object_hash_excluding_work, top_u64_be, Object, ObjectInput};
use pow_program::{PowIn, PowOut};
use stone_program::constants::STONE_MINING_MAX;

mod save;

const POW_ELF: &[u8] = include_elf!("pow-program");
const STONE_ELF: &[u8] = include_elf!("stone-program");

fn mine_stone_object() -> (Object, [u8; 32]) {
    let key = {
        let bytes: [u8; 32] = rand::random();
        hex::encode(bytes)
    };

    for seed in 0u32..=u32::MAX {
        let obj = Object {
            key: key.clone(),
            inputs: vec![],
            seed,
            blueprint: "stone".to_string(),
            work: [0u8; 32], // placeholder for now
        };

        let h = object_hash_excluding_work(&obj);
        if top_u64_be(h) <= STONE_MINING_MAX {
            return (obj, h);
        }
    }

    panic!("failed to mine stone object");
}

fn main() {
    // Setup logging.
    utils::setup_logger();

    let client = ProverClient::from_env();

    // Setup keys.
    let (pow_pk, pow_vk) = client.setup(POW_ELF);
    let (stone_pk, stone_vk) = client.setup(STONE_ELF);
    println!("pow vk hash_u32 = {:?}", pow_vk.hash_u32());

    // 1) Mine (key, seed) to satisfy difficulty.
    let (mut obj, obj_hash) = mine_stone_object();
    println!("mined stone: obj={:?}, obj_hash={:?}", obj, obj_hash);

    // 2) Prove PoW with n_iters = 3, input = obj_hash
    let mut pow_stdin = SP1Stdin::new();
    pow_stdin.write(&PowIn {
        n_iters: 3,
        input: obj_hash,
    });

    let mut pow_proof: SP1ProofWithPublicValues = client
        .prove(&pow_pk, &pow_stdin)
        .compressed()
        .run()
        .expect("pow proving failed");

    client
        .verify(&pow_proof, &pow_vk)
        .expect("pow verify failed");

    // Read PowOut (this must match what your guest reads / bincode-digests).
    let pow_out: PowOut = pow_proof.public_values.read();
    println!("pow_out = {:?}", pow_out);

    // Set the object's work to the pow output.
    obj.work = pow_out.output;

    // 3) Prove stone program
    // stone guest reads: ObjectInput, then PowOut, then calls verify_sp1_proof (reads proof from proof stream)
    let mut stone_stdin = SP1Stdin::new();

    stone_stdin.write(&ObjectInput {
        hash: obj_hash,
        object: obj.clone(),
    });
    stone_stdin.write(&pow_out);

    let SP1Proof::Compressed(pow_proof) = pow_proof.proof else {
        panic!()
    };
    stone_stdin.write_proof(*pow_proof, pow_vk.clone().vk);

    let mut stone_proof: SP1ProofWithPublicValues = client
        .prove(&stone_pk, &stone_stdin)
        .compressed()
        .run()
        .expect("stone proving failed");

    client
        .verify(&stone_proof, &stone_vk)
        .expect("stone verify failed");

    let committed_hash: [u8; 32] = stone_proof.public_values.read();
    println!("stone committed hash = {:?}", committed_hash);

    save::save_object_as_json(&obj, &stone_proof, "stone.json").expect("failed to save stone");
    println!("saved stone to stone.json");
}
