use dotenvy::EnvLoader;

use alloy::{
    consensus::{SidecarBuilder, SimpleCoder},
    eips::eip4844::DATA_GAS_PER_BLOB,
    network::{TransactionBuilder, TransactionBuilder4844},
    primitives::TxHash,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};

pub async fn send_blob_tx(blob_data: &[u8]) -> Result<TxHash, Box<dyn std::error::Error>> {
    let env_map = EnvLoader::new().load()?;

    let signer: PrivateKeySigner = env_map.var("PRIVATE_KEY")?.parse();
    let provider = ProviderBuilder::new()
        .wallet(signer.clone())
        .connect(env_map.var("RPC_URL")?)
        .await?;
    let latest_block = provider.get_block_number().await?;
    println!("Latest block number: {latest_block}");
    let sender = signer.address();
    let receiver = env_map.var("TO_ADDRESS")?.parse()?;
    println!("Sender address: {sender}");
    println!("Receiver address: {receiver}");

    // Create a sidecar with some data.
    let sidecar: SidecarBuilder<SimpleCoder> = SidecarBuilder::from_slice(blob_data);
    let sidecar = sidecar.build()?;

    // The `from` field is automatically filled to the first signer's address (Alice).
    let tx = TransactionRequest::default()
        .with_to(receiver)
        .with_blob_sidecar(sidecar);

    // Send the transaction and wait for the broadcast.
    let pending_tx = provider.send_transaction(tx).await?;

    println!("Pending transaction... {}", pending_tx.tx_hash());

    // Wait for the transaction to be included and get the receipt.
    let receipt = pending_tx.get_receipt().await?;

    println!(
        "Transaction included in block {}",
        receipt.block_number.expect("Failed to get block number")
    );

    assert_eq!(receipt.from, sender);
    assert_eq!(receipt.to, Some(receiver));
    assert_eq!(
        receipt
            .blob_gas_used
            .expect("Expected to be EIP-4844 transaction"),
        DATA_GAS_PER_BLOB
    );

    Ok(receipt.transaction_hash)
}
