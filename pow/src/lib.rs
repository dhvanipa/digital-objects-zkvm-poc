use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct PowIn {
    pub n_iters: u32,
    pub input: [u8; 32],
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct PowOut {
    pub n_iters: u32,
    pub input: [u8; 32],
    pub output: [u8; 32],
}
