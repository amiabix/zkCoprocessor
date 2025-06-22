use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::path::Path;
use tigerbeetle_unofficial::Client;
use tracing::{info, warn};
use hex;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub transfer_id: u128,
    pub block_number: u64,
    pub tx_index: usize,
    pub from_account: u128,
    pub to_account: u128,
    pub amount: u128,
    pub tx_hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionProof {
    pub transfer_id: u128,
    pub block_number: u64,
    pub inclusion_proof_hash: [u8; 32],
    pub is_valid: bool,
    pub timestamp: u64,
    pub proof_path: Option<String>,
    pub proof_type: String, // "zisk" or "simulated"
}

/// Generate ZK proof for a transaction using ZisK (with Mac fallback)
pub async fn generate_zk_proof(
    tb_client: &mut Client,
    transfer_id: u128,
) -> Result<TransactionProof> {
    info!("🎯 Generating ZK proof for transfer_id: {}", transfer_id);
    
    // 1. Fetch transaction data from TigerBeetle
    let transfer_data = fetch_transfer_data(tb_client, transfer_id).await?;
    
    // 2. Check if ZisK is available and supported on this platform
    if is_zisk_available() && is_platform_supported() {
        // Use real ZisK proof generation
        match generate_zisk_proof(&transfer_data).await {
            Ok(proof) => {
                info!("✅ Real ZisK proof generated: valid={}", proof.is_valid);
                return Ok(proof);
            }
            Err(e) => {
                warn!("⚠️  ZisK proof failed, falling back to simulation: {}", e);
            }
        }
    } else {
        if !is_platform_supported() {
            info!("ℹ️  ZisK doesn't support {} yet", std::env::consts::OS);
        } else {
            info!("ℹ️  ZisK not available, using simulation mode");
        }
    }
    
    // 3. Fallback to simulated proof
    let proof = generate_simulated_proof(&transfer_data).await?;
    info!("✅ Simulated proof generated: valid={}", proof.is_valid);
    Ok(proof)
}

/// Check if ZisK is properly installed
fn is_zisk_available() -> bool {
    match Command::new("cargo-zisk").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                info!("✅ ZisK is available");
                true
            } else {
                info!("❌ ZisK command failed");
                false
            }
        }
        Err(_) => {
            info!("❌ ZisK not found in PATH");
            false
        }
    }
}

/// Check if current platform is supported by ZisK
fn is_platform_supported() -> bool {
    // Temporarily allow macOS for testing our ZisK integration
    // In production, this should be: let supported = os != "macos";
    let os = std::env::consts::OS;
    let supported = true; // Allow all platforms for testing
    
    if !supported {
        info!("⚠️  ZisK doesn't support {} yet", os);
    }
    
    supported
}

/// Generate proof using real ZisK zkVM (when supported)
async fn generate_zisk_proof(transfer_data: &TransactionData) -> Result<TransactionProof> {
    info!("🚀 Generating real ZisK proof...");
    
    // 1. Build the ZisK program if not already built
    build_zisk_program().await?;
    
    // 2. Run the ZisK program with the transaction data
    let proof_result = run_zisk_program(transfer_data).await?;
    
    // 3. Parse the proof results
    let proof = parse_zisk_proof_result(&proof_result, transfer_data)?;
    
    Ok(proof)
}

/// Run the ZisK program with transaction data
async fn run_zisk_program(transfer_data: &TransactionData) -> Result<String> {
    info!("🎯 Running ZisK program with transfer_id: {}", transfer_data.transfer_id);
    
    // Log the data being passed to ZisK
    info!("📊 Data from TigerBeetle being passed to ZisK:");
    info!("   Transfer ID: {}", transfer_data.transfer_id);
    info!("   Block Number: {}", transfer_data.block_number);
    info!("   TX Index: {}", transfer_data.tx_index);
    info!("   From Account: {}", transfer_data.from_account);
    info!("   To Account: {}", transfer_data.to_account);
    info!("   Amount: {} wei", transfer_data.amount);
    info!("   TX Hash: {}", hex::encode(transfer_data.tx_hash));
    
    // Change to zisk-tx-proof directory
    let current_dir = std::env::current_dir()?;
    let zisk_dir = current_dir.join("zisk-tx-proof");
    
    // Create input.bin file with transaction data
    let input_file = zisk_dir.join("build").join("input.bin");
    
    // Ensure build directory exists
    if let Some(parent) = input_file.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Write transaction data to input.bin in little-endian format
    let mut file = fs::File::create(&input_file)?;
    file.write_all(&transfer_data.transfer_id.to_le_bytes())?;
    file.write_all(&transfer_data.block_number.to_le_bytes())?;
    file.write_all(&(transfer_data.tx_index as u64).to_le_bytes())?;
    file.write_all(&transfer_data.from_account.to_le_bytes())?;
    file.write_all(&transfer_data.to_account.to_le_bytes())?;
    file.write_all(&transfer_data.amount.to_le_bytes())?;
    
    info!("📝 Created input.bin with transaction data");
    
    // Run the ZisK program using cargo
    let output = Command::new("cargo")
        .args(&["run"])
        .current_dir(&zisk_dir)
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "ZisK program failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    info!("✅ ZisK program completed successfully");
    info!("ZisK output: {}", stdout);
    
    // Clean up input file
    cleanup_temp_files(&input_file.to_string_lossy())?;
    
    Ok(stdout.to_string())
}

/// Parse the ZisK program output to extract proof results
fn parse_zisk_proof_result(output: &str, transfer_data: &TransactionData) -> Result<TransactionProof> {
    info!("📊 Parsing ZisK proof results...");
    
    // Extract the public outputs from the ZisK output
    // The output format is: "public 0: 0x00000014" etc.
    let mut public_outputs = Vec::new();
    
    for line in output.lines() {
        if line.contains("public") && line.contains("0x") {
            if let Some(hex_part) = line.split("0x").nth(1) {
                if let Some(value_str) = hex_part.split_whitespace().next() {
                    if let Ok(value) = u32::from_str_radix(value_str, 16) {
                        public_outputs.push(value);
                    }
                }
            }
        }
    }
    
    info!("📊 Found {} public outputs: {:?}", public_outputs.len(), public_outputs);
    
    // Parse the public outputs back into our proof structure
    // Based on our ZisK program's write_transaction_proof function:
    // - outputs 0-3: transfer_id (4 u32 values)
    // - outputs 4-5: block_number (2 u32 values) 
    // - outputs 6-7: inclusion_proof_hash (first 8 bytes as 2 u32 values)
    // - output 8: validity (0 or 1)
    
    if public_outputs.len() < 9 {
        return Err(anyhow::anyhow!("Insufficient public outputs from ZisK program"));
    }
    
    // Reconstruct transfer_id from 4 u32 values
    let transfer_id_bytes = [
        (public_outputs[0] & 0xFF) as u8,
        ((public_outputs[0] >> 8) & 0xFF) as u8,
        ((public_outputs[0] >> 16) & 0xFF) as u8,
        ((public_outputs[0] >> 24) & 0xFF) as u8,
        (public_outputs[1] & 0xFF) as u8,
        ((public_outputs[1] >> 8) & 0xFF) as u8,
        ((public_outputs[1] >> 16) & 0xFF) as u8,
        ((public_outputs[1] >> 24) & 0xFF) as u8,
        (public_outputs[2] & 0xFF) as u8,
        ((public_outputs[2] >> 8) & 0xFF) as u8,
        ((public_outputs[2] >> 16) & 0xFF) as u8,
        ((public_outputs[2] >> 24) & 0xFF) as u8,
        (public_outputs[3] & 0xFF) as u8,
        ((public_outputs[3] >> 8) & 0xFF) as u8,
        ((public_outputs[3] >> 16) & 0xFF) as u8,
        ((public_outputs[3] >> 24) & 0xFF) as u8,
    ];
    let transfer_id = u128::from_le_bytes(transfer_id_bytes);
    
    // Reconstruct block_number from 2 u32 values
    let block_number_bytes = [
        (public_outputs[4] & 0xFF) as u8,
        ((public_outputs[4] >> 8) & 0xFF) as u8,
        ((public_outputs[4] >> 16) & 0xFF) as u8,
        ((public_outputs[4] >> 24) & 0xFF) as u8,
        (public_outputs[5] & 0xFF) as u8,
        ((public_outputs[5] >> 8) & 0xFF) as u8,
        ((public_outputs[5] >> 16) & 0xFF) as u8,
        ((public_outputs[5] >> 24) & 0xFF) as u8,
    ];
    let block_number = u64::from_le_bytes(block_number_bytes);
    
    // Reconstruct inclusion_proof_hash (first 8 bytes)
    let mut inclusion_proof_hash = [0u8; 32];
    inclusion_proof_hash[0] = (public_outputs[6] & 0xFF) as u8;
    inclusion_proof_hash[1] = ((public_outputs[6] >> 8) & 0xFF) as u8;
    inclusion_proof_hash[2] = ((public_outputs[6] >> 16) & 0xFF) as u8;
    inclusion_proof_hash[3] = ((public_outputs[6] >> 24) & 0xFF) as u8;
    inclusion_proof_hash[4] = (public_outputs[7] & 0xFF) as u8;
    inclusion_proof_hash[5] = ((public_outputs[7] >> 8) & 0xFF) as u8;
    inclusion_proof_hash[6] = ((public_outputs[7] >> 16) & 0xFF) as u8;
    inclusion_proof_hash[7] = ((public_outputs[7] >> 24) & 0xFF) as u8;
    
    // Validity flag
    let is_valid = public_outputs[8] == 1;
    
    info!("📊 Parsed proof - transfer_id: {}, block_number: {}, valid: {}", 
          transfer_id, block_number, is_valid);
    
    Ok(TransactionProof {
        transfer_id,
        block_number,
        inclusion_proof_hash,
        is_valid,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        proof_path: None,
        proof_type: "zisk".to_string(),
    })
}

/// Build the ZisK program
async fn build_zisk_program() -> Result<()> {
    info!("🔨 Building ZisK program...");
    
    let output = Command::new("rustup")
        .args(&["run", "zisk", "cargo", "build", "--release"])
        .current_dir("./zisk-tx-proof")
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to build ZisK program: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    info!("✅ ZisK program built successfully");
    Ok(())
}

/// Cleanup temporary files
fn cleanup_temp_files(input_file: &str) -> Result<()> {
    if Path::new(input_file).exists() {
        fs::remove_file(input_file)?;
    }
    Ok(())
}

/// Simulated proof generation (fallback)
async fn generate_simulated_proof(transfer_data: &TransactionData) -> Result<TransactionProof> {
    // Basic validation
    let is_valid = transfer_data.amount > 0 
        && transfer_data.from_account != transfer_data.to_account;
    
    let inclusion_proof_hash = compute_inclusion_hash(transfer_data);
    
    Ok(TransactionProof {
        transfer_id: transfer_data.transfer_id,
        block_number: transfer_data.block_number,
        inclusion_proof_hash,
        is_valid,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        proof_path: None,
        proof_type: "simulated".to_string(),
    })
}

/// Fetch transaction data from TigerBeetle
async fn fetch_transfer_data(
    tb_client: &mut Client,
    transfer_id: u128,
) -> Result<TransactionData> {
    info!("🔍 Fetching transfer data from TigerBeetle for ID: {}", transfer_id);
    
    let transfers = tb_client.lookup_transfers(vec![transfer_id]).await?;
    
    if transfers.is_empty() {
        return Err(anyhow::anyhow!("Transfer {} not found", transfer_id));
    }
    
    let transfer = &transfers[0];
    let block_number = transfer.user_data_128() as u64;
    let tx_index = (transfer_id % 1000000) as usize;
    
    let mut tx_hash = [0u8; 32];
    let id_bytes = transfer_id.to_le_bytes();
    tx_hash[0..16].copy_from_slice(&id_bytes);
    
    info!("📊 Raw data from TigerBeetle:");
    info!("   Transfer ID: {}", transfer_id);
    info!("   Block Number: {} (from user_data_128)", block_number);
    info!("   TX Index: {} (calculated from transfer_id % 1000000)", tx_index);
    info!("   From Account: {} (debit_account_id)", transfer.debit_account_id());
    info!("   To Account: {} (credit_account_id)", transfer.credit_account_id());
    info!("   Amount: {} wei (amount field)", transfer.amount());
    info!("   TX Hash: {} (derived from transfer_id)", hex::encode(tx_hash));
    
    Ok(TransactionData {
        transfer_id,
        block_number,
        tx_index,
        from_account: transfer.debit_account_id(),
        to_account: transfer.credit_account_id(),
        amount: transfer.amount(),
        tx_hash,
    })
}

/// Compute inclusion hash
fn compute_inclusion_hash(transfer_data: &TransactionData) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    
    let mut hasher = Sha256::new();
    hasher.update(&transfer_data.transfer_id.to_le_bytes());
    hasher.update(&transfer_data.block_number.to_le_bytes());
    hasher.update(&(transfer_data.tx_index as u64).to_le_bytes());
    hasher.update(&transfer_data.from_account.to_le_bytes());
    hasher.update(&transfer_data.to_account.to_le_bytes());
    hasher.update(&transfer_data.amount.to_le_bytes());
    
    hasher.finalize().into()
}