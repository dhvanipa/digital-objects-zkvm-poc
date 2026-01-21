use common::ObjectHash;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ObjectOutputWithType {
    pub hash: ObjectHash,
    pub consumed: Vec<ObjectHash>,
    pub blueprint: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CommitIn {
    pub objects: Vec<ObjectOutputWithType>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CommitOut {
    pub created: Vec<ObjectHash>,
    pub consumed: Vec<ObjectHash>,
}
