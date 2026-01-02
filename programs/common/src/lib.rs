use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type ObjectHash = String;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Object {
    pub key: String,
    pub inputs: Vec<ObjectHash>,
    pub seed: u32,
    pub blueprint: String, // TODO: change to enum
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

pub fn hex_to_vk_digest(hex_str: &str) -> [u32; 8] {
    bytes_to_words_be(
        hex::decode(hex_str)
            .expect("valid hex string")
            .as_slice()
            .try_into()
            .expect("32 bytes for vkey hash"),
    )
}

/// Utility method for converting 32 big-endian bytes back into eight u32 words.
fn bytes_to_words_be(bytes: &[u8; 32]) -> [u32; 8] {
    let mut words = [0u32; 8];
    for i in 0..8 {
        let chunk: [u8; 4] = bytes[i * 4..(i + 1) * 4].try_into().unwrap();
        words[i] = u32::from_be_bytes(chunk);
    }
    words
}
