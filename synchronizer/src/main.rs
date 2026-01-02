use std::{
    collections::{HashMap, HashSet},
    fs::{create_dir_all, read_dir, rename, File},
    io,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use commit_program::CommitOut;
use common::ObjectHash;
use sp1_sdk::{include_elf, utils, EnvProver, HashableKey, ProverClient};
use synchronizer::{
    bytes_from_simple_blob,
    clients::beacon::{
        self,
        types::{Blob, BlockHeader, BlockId},
        BeaconClient,
    },
};

use ::utils::load_proof_from_json_file;
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
use chrono::{DateTime, Utc};
use tracing::{debug, info, trace};

const COMMIT_ELF: &[u8] = include_elf!("commit-program");

#[derive(Debug)]
struct State {
    created_items: HashSet<ObjectHash>,
    consumed_items: HashSet<ObjectHash>,
}

struct Node {
    spclient: EnvProver,
    commit_vk: sp1_sdk::SP1VerifyingKey,
    beacon_cli: BeaconClient,
    rpc_cli: RootProvider,
    // Mutable state
    state: RwLock<State>,
}

// This node code is adapted from https://github.com/0xPARC/digital-objects-e2e-poc/blob/main/synchronizer/src/main.rs
impl Node {
    async fn new() -> Result<Self> {
        let spclient = ProverClient::from_env();

        println!("Setting up proving/verifying keys...");
        let (commit_pk, commit_vk) = spclient.setup(COMMIT_ELF);
        println!("commit program vk {:?}", commit_vk.hash_u32());

        let http_cli = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()?;
        let rpc_url: String = dotenvy::var("RPC_URL")?;
        let beacon_url: String = dotenvy::var("BEACON_URL")?;

        let exp_backoff = Some(ExponentialBackoffBuilder::default().build());
        let beacon_cli_cfg = beacon::Config {
            base_url: beacon_url.clone(),
            exp_backoff,
        };
        let beacon_cli = BeaconClient::try_with_client(http_cli, beacon_cli_cfg)?;
        let rpc_cli = RootProvider::<Ethereum>::new_http(rpc_url.parse()?);

        let state = State {
            created_items: HashSet::new(),
            consumed_items: HashSet::new(),
        };
        Ok(Self {
            spclient,
            commit_vk,
            beacon_cli,
            rpc_cli,
            state: RwLock::new(state),
        })
    }

    fn slot_dir(&self, slot: u32) -> PathBuf {
        let slot_hi = slot / 1_000_000;
        let slot_mid = (slot - slot_hi * 1_000_000) / 1_000;
        let slot_lo = slot - slot_hi * 1_000_000 - slot_mid * 1_000;
        let slot_dir: PathBuf = [
            &dotenvy::var("BLOBS_PATH").expect("blobs path expected"),
            &format!("{:03}", slot_hi),
            &format!("{:03}", slot_mid),
            &format!("{:03}", slot_lo),
        ]
        .iter()
        .collect();
        slot_dir
    }

    async fn load_blobs_disk(&self, slot: u32) -> Result<HashMap<B256, Blob>> {
        let slot_dir = self.slot_dir(slot);
        let rd = match read_dir(&slot_dir) {
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    return Ok(HashMap::new());
                } else {
                    return Err(e.into());
                }
            }
            Ok(rd) => rd,
        };
        debug!("loading blobs of slot {} from {:?}", slot, slot_dir);
        let mut blobs = HashMap::new();
        for entry in rd {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap_or("");
            if file_name.starts_with("blob-") && file_name.ends_with(".cbor") {
                let file_path = slot_dir.join(file_name);
                let mut file = File::open(&file_path)?;
                let mut data_cbor = Vec::new();
                file.read_to_end(&mut data_cbor)?;
                let blob: Blob = minicbor_serde::from_slice(&data_cbor)?;
                let versioned_hash = kzg_to_versioned_hash(blob.kzg_commitment.as_ref());
                blobs.insert(versioned_hash, blob);
            }
        }
        Ok(blobs)
    }

    async fn store_blobs_disk(&self, slot: u32, blobs: &HashMap<B256, Blob>) -> Result<()> {
        let slot_dir = self.slot_dir(slot);
        debug!("storing blobs of slot {} to {:?}", slot, slot_dir);
        create_dir_all(&slot_dir)?;
        for (vh, blob) in blobs {
            let name = format!("blob-{}.cbor", vh);
            let blob_path = slot_dir.join(&name);
            let blob_path_tmp = slot_dir.join(format!("{}.tmp", name));
            let mut file_tmp = File::create(&blob_path_tmp)?;
            let blob_cbor = minicbor_serde::to_vec(blob)?;
            file_tmp.write_all(&blob_cbor)?;
            rename(blob_path_tmp, blob_path)?;
        }
        Ok(())
    }

    // Checks that the blobs contain all the blobs identified by `versioned_hashes`.  If some are
    // missing, return the versioned_hash of the first missing one.
    fn validate_blobs(blobs: &HashMap<B256, Blob>, versioned_hashes: &[B256]) -> Option<B256> {
        for vh in versioned_hashes.iter() {
            if !blobs.contains_key(vh) {
                return Some(*vh);
            }
        }
        None
    }

    async fn get_blobs(&self, slot: u32, versioned_hashes: &[B256]) -> Result<HashMap<B256, Blob>> {
        let blobs = self.load_blobs_disk(slot).await?;
        if Self::validate_blobs(&blobs, versioned_hashes).is_some() {
            let blobs = self.beacon_cli.get_blobs(slot.into()).await?;
            debug!("got {} DO blobs from beacon_cli", blobs.len());
            let blobs: HashMap<_, _> = blobs
                .into_iter()
                .filter_map(|blob| {
                    let versioned_hash = kzg_to_versioned_hash(blob.kzg_commitment.as_ref());
                    versioned_hashes
                        .contains(&versioned_hash)
                        .then_some((versioned_hash, blob))
                })
                .collect();
            if let Some(vh) = Self::validate_blobs(&blobs, versioned_hashes) {
                return Err(anyhow!("Blob {} not found in beacon_cli response", vh));
            }
            self.store_blobs_disk(slot, &blobs).await?;
            Ok(blobs)
        } else {
            Ok(blobs)
        }
    }

    async fn process_beacon_block_header(
        &self,
        beacon_block_header: &BlockHeader,
    ) -> Result<Option<()>> {
        let beacon_block_root = beacon_block_header.root;
        let slot = beacon_block_header.slot;

        let beacon_block = match self
            .beacon_cli
            .get_block(BlockId::Hash(beacon_block_root))
            .await?
        {
            Some(block) => block,
            None => {
                debug!("slot {} has empty block", slot);
                return Ok(None);
            }
        };
        let execution_payload = match beacon_block.execution_payload {
            Some(payload) => payload,
            None => {
                debug!("slot {} has no execution payload", slot);
                return Ok(None);
            }
        };
        debug!(
            "slot {} has execution block {} at height {}",
            slot, execution_payload.block_hash, execution_payload.block_number
        );

        info!(
            "processing slot {} from {}",
            slot,
            DateTime::<Utc>::from_timestamp_secs(execution_payload.timestamp as i64)
                .unwrap_or_default(),
        );

        let has_kzg_blob_commitments = match beacon_block.blob_kzg_commitments {
            Some(commitments) => !commitments.is_empty(),
            None => false,
        };
        if !has_kzg_blob_commitments {
            debug!("slot {} has no blobs", slot);
            return Ok(None);
        }

        let execution_block_hash = execution_payload.block_hash;

        let execution_block_id = alloy_eips::eip1898::BlockId::Hash(execution_block_hash.into());
        let execution_block = self
            .rpc_cli
            .get_block(execution_block_id)
            .full()
            .await?
            .with_context(|| format!("Execution block {execution_block_hash} not found"))?;

        let to_addr: Address = Address::from_str(&dotenvy::var("TO_ADDRESS")?)?;
        let indexed_do_blob_txs: Vec<_> = match execution_block.transactions.as_transactions() {
            Some(txs) => txs
                .iter()
                .enumerate()
                .filter(|(_index, tx)| {
                    tx.inner.blob_versioned_hashes().is_some()
                        && tx.as_recovered().to() == Some(to_addr)
                })
                .collect(),
            None => {
                return Err(anyhow!(
                    "Consensus block {beacon_block_root} has blobs but the execution block doesn't have txs"
                ));
            }
        };

        if indexed_do_blob_txs.is_empty() {
            return Ok(None);
        }

        let txs_blobs_vhs: Vec<B256> = indexed_do_blob_txs
            .iter()
            .flat_map(|(_, tx)| {
                tx.as_recovered()
                    .blob_versioned_hashes()
                    .expect("tx has blobs")
            })
            .cloned()
            .collect();
        let blobs = self.get_blobs(slot, &txs_blobs_vhs).await?;

        for (_tx_index, tx) in indexed_do_blob_txs {
            let tx = tx.as_recovered();
            let hash = tx.hash();
            let from = tx.signer();
            let to = tx.to();
            let tx_blobs: Vec<_> = tx
                .blob_versioned_hashes()
                .expect("tx has blobs")
                .iter()
                .map(|blob_versioned_hash| &blobs[blob_versioned_hash])
                .collect();
            trace!(?hash, ?from, ?to);

            for blob in tx_blobs.iter() {
                match self.process_do_blob(blob).await {
                    Ok(_) => {
                        info!("Valid do_blob at slot {}, blob_index {}!", slot, blob.index);
                    }
                    Err(e) => {
                        info!("Invalid do_blob: {:?}", e);
                        continue;
                    }
                };
            }
        }
        Ok(Some(()))
    }

    async fn process_do_blob(&self, blob: &Blob) -> Result<()> {
        let bytes =
            bytes_from_simple_blob(blob.blob.inner()).context("Invalid byte encoding in blob")?;
        // let payload = Payload::from_bytes(&bytes, &self.common_circuit_data)?;
        let commit_proof_hash = hex::encode(bytes);
        info!("Processing commitment {}", commit_proof_hash);

        // Read this file from commitments/
        let commit_proof =
            load_proof_from_json_file(&format!("commitments/{}.json", commit_proof_hash))
                .expect("Expect commitment file to exist");
        self.spclient
            .verify(&commit_proof, &self.commit_vk)
            .expect("commit verify failed");
        let commit_out: CommitOut = commit_proof.public_values.clone().read();
        let mut state = self.state.write().expect("lock");

        // Check that output is unique
        for item in &commit_out.created {
            if state.created_items.contains(item) {
                bail!("item {} exists in created_items", item);
            }
        }

        // Check that inputs are unique
        for item in &commit_out.consumed {
            if !state.created_items.contains(item) {
                bail!("item {} doesn't exist in created_items", item);
            }

            if state.consumed_items.contains(item) {
                bail!("item {} exists in consumed_items", item);
            }
        }

        // Register items
        for item in &commit_out.created {
            state.created_items.insert(item.clone());
        }
        for item in &commit_out.consumed {
            state.consumed_items.insert(item.clone());
        }

        info!(
            "state update: created_items={:?}, consuemd_items={:?}, ",
            state.created_items, state.consumed_items,
        );
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    utils::setup_logger();

    let node = Arc::new(Node::new().await?);

    let spec = node.beacon_cli.get_spec().await?;
    info!(?spec, "Beacon spec");
    let mut head = node
        .beacon_cli
        .get_block_header(BlockId::Head)
        .await?
        .expect("head is not None");
    info!(?head, "Beacon head");

    let mut slot = head.slot;
    loop {
        debug!("checking slot {}", slot);
        let some_beacon_block_header = if slot <= head.slot {
            node.beacon_cli
                .get_block_header(BlockId::Slot(slot))
                .await?
        } else {
            // TODO: Be more fancy and replace this with a stream from an event subscription to
            // Beacon Headers
            tokio::time::sleep(Duration::from_secs(5)).await;
            loop {
                head = node
                    .beacon_cli
                    .get_block_header(BlockId::Head)
                    .await?
                    .expect("head is not None");
                if head.slot > slot {
                    debug!(
                        "head is {}, slot {} was skipped, retrieving...",
                        head.slot, slot
                    );
                    break node
                        .beacon_cli
                        .get_block_header(BlockId::Slot(slot))
                        .await?;
                } else if head.slot == slot {
                    break Some(head.clone());
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        };
        let beacon_block_header = match some_beacon_block_header {
            Some(block) => block,
            None => {
                debug!("slot {} has empty block", slot);
                slot += 1;
                continue;
            }
        };

        node.process_beacon_block_header(&beacon_block_header)
            .await?;
        let request_rate = 15;

        let requests = 5;
        let delay_ms = 1000 * requests / request_rate;
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;

        slot += 1;
    }
}
