use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Object {
    pub key: String,
    pub inputs: Vec<String>,
    pub seed: u32,
    pub blueprint: String,
    pub work: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ObjectInput {
    pub hash: [u8; 32],
    pub object: Object,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct ObjectOutput {
    pub hash: [u8; 32],
}

pub fn object_hash_excluding_work(obj: &Object) -> [u8; 32] {
    let mut o = obj.clone();
    o.work = [0u8; 32];
    let bytes = bincode::serialize(&o).expect("serialize Object");
    Sha256::digest(&bytes).into()
}

pub fn top_u64_be(hash: [u8; 32]) -> u64 {
    u64::from_be_bytes(hash[0..8].try_into().unwrap())
}
