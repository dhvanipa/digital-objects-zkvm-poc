use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type ObjectHash = [u8; 32];

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Object {
    pub key: String,
    pub inputs: Vec<ObjectHash>,
    pub seed: u32,
    pub blueprint: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ObjectInput {
    pub object: Object,
    pub work: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ObjectOutput {
    pub hash: ObjectHash,
    pub consumed: Vec<ObjectHash>,
}

impl Object {
    pub fn hash(&self) -> [u8; 32] {
        let bytes = bincode::serialize(&self.clone()).expect("serialize Object");
        Sha256::digest(&bytes).into()
    }
}

pub fn top_u64_be(hash: [u8; 32]) -> u64 {
    u64::from_be_bytes(hash[0..8].try_into().unwrap())
}
