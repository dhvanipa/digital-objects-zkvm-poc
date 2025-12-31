use serde::{Deserialize, Serialize};

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
