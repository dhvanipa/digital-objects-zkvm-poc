use sp1_sdk::SP1ProofWithPublicValues;

pub fn save_proof_as_json(
    proof: &SP1ProofWithPublicValues,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(proof)?;
    let mut file = File::create(path.as_ref())?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

pub fn load_proof_from_json(
    path: impl AsRef<Path>,
) -> Result<SP1ProofWithPublicValues, Box<dyn std::error::Error>> {
    let file = File::open(path.as_ref())?;
    let proof = serde_json::from_reader(file)?;
    Ok(proof)
}
