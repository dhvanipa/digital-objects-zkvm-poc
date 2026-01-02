use std::time::Duration;

use ::utils::{save_proof_as_json, ObjectJson};
use commit_program::{CommitIn, CommitOut, ObjectOutputWithType};
use common::ObjectOutput;
use sha2::{Digest, Sha256};
use sp1_sdk::{
    include_elf, utils, EnvProver, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues,
    SP1Stdin,
};
use synchronizer::clients::beacon::{self, types::BlockId, BeaconClient};

use alloy::{
    consensus::Transaction,
    eips::{self as alloy_eips, eip4844::kzg_to_versioned_hash},
    network as alloy_network,
    primitives::{Address, B256},
    providers as alloy_provider,
    transports::http::reqwest,
};
use alloy_network::Ethereum;
use alloy_provider::{Provider, RootProvider};
use anyhow::{anyhow, bail, Context, Result};
use backoff::ExponentialBackoffBuilder;

const COMMIT_ELF: &[u8] = include_elf!("commit-program");

#[tokio::main]
async fn main() -> Result<()> {
    utils::setup_logger();

    let client = ProverClient::from_env();

    println!("Setting up proving/verifying keys...");
    let (commit_pk, commit_vk) = client.setup(COMMIT_ELF);
    println!("commit program vk {:?}", commit_vk.hash_u32());

    let http_cli = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()?;

    let exp_backoff = Some(ExponentialBackoffBuilder::default().build());
    let beacon_cli_cfg = beacon::Config {
        base_url: dotenvy::var("BEACON_URL")?.clone(),
        exp_backoff,
    };
    let beacon_cli = BeaconClient::try_with_client(http_cli, beacon_cli_cfg)?;

    let spec = beacon_cli.get_spec().await?;
    println!("Beacon spec: {:?}", spec);
    let mut head = beacon_cli
        .get_block_header(BlockId::Head)
        .await?
        .expect("head is not None");
    println!("Beacon head: {:?}", head);

    Ok(())
}
