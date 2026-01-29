use common::{Object, ObjectHash};
use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::Path};

#[derive(Serialize, Deserialize)]
pub struct ObjectJson {
    pub object: Object,
    pub hash: ObjectHash,
    pub work: String,
    pub proof: Receipt,
    pub program_vk: [u32; 8],
}

impl ObjectJson {
    pub fn save_as_json(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(path.as_ref())?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn save_as_bytes(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = bincode::serialize(&self)?;
        let mut file = File::create(path.as_ref())?;
        file.write_all(&bytes)?;
        Ok(())
    }

    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path.as_ref())?;
        let object_json: ObjectJson = serde_json::from_reader(file)?;
        Ok(object_json)
    }
}

pub fn save_proof_as_json(
    proof: &Receipt,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(&proof)?;
    let mut file = File::create(path.as_ref())?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn load_proof_from_json_file(
    path: impl AsRef<Path>,
) -> Result<Receipt, Box<dyn std::error::Error>> {
    let file = File::open(path.as_ref())?;
    let proof: Receipt = serde_json::from_reader(file)?;
    Ok(proof)
}
