use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct PV {
    pub count: u32,
    pub base_input: [u8; 32],
    pub x: [u8; 32],
}
