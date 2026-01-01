use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PowIn {
    pub n_iters: u32,
    pub input: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PowOut {
    pub n_iters: u32,
    pub input: String,
    pub output: String,
}
