use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type ObjectHash = String;

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
    pub work: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ObjectOutput {
    pub hash: ObjectHash,
    pub consumed: Vec<ObjectHash>,
}

impl Object {
    pub fn hash(&self) -> String {
        let bytes = bincode::serialize(&self.clone()).expect("serialize Object");
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        hex::encode(digest)
    }
}

pub fn difficulty(hash: &str) -> u64 {
    let bytes = hex::decode(hash).expect("valid hex hash");
    u64::from_be_bytes(bytes[0..8].try_into().unwrap())
}
