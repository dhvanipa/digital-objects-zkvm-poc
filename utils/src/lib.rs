use common::{Object, ObjectHash};
use serde::Serialize;
use sp1_sdk::SP1ProofWithPublicValues;
use std::{fs::File, io::Write, path::Path};

#[derive(Serialize)]
pub struct ObjectJson {
    pub object: Object,
    pub hash: ObjectHash,
    pub work: String,
    pub proof: SP1ProofWithPublicValues,
}

impl ObjectJson {
    pub fn save_as_json(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(path.as_ref())?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
