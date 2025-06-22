use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::path::Path;
use tigerbeetle_unofficial::Client;
use tracing::{info, warn};

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
            info!("ℹ️  ZisK not supported on macOS yet, using simulation mode");
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
    // ZisK currently doesn't support macOS
    let os = std::env::consts::OS;
    let supported = os != "macos";
    
    if !supported {
        info!("⚠️  ZisK doesn't support {} yet", os);
    }
    
    supported
}

/// Generate proof using real ZisK zkVM (when supported)
async fn generate_zisk_proof(transfer_data: &TransactionData) -> Result<TransactionProof> {
    info!("🚀 Generating real ZisK proof...");
    
    // 1. Prepare JSON input for ZisK program
    let input_file = prepare_zisk_input_json(transfer_data)?;
    
    // 2. Build the ZisK program
    build_zisk_program().await?;
    
    // 3. Test with emulator first
    test_zisk_program(&input_file).await?;
    
    // 4. Generate the actual proof
    let proof_dir = generate_zisk_proof_files(&input_file).await?;
    
    // 5. Parse the proof results
    let proof = parse_zisk_proof_output(&proof_dir, transfer_data)?;
    
    // 6. Cleanup
    cleanup_temp_files(&input_file)?;
    
    Ok(proof)
}

/// Prepare JSON input file for ZisK program
fn prepare_zisk_input_json(transfer_data: &TransactionData) -> Result<String> {
    let input_file = format!("./zisk-tx-proof/build/input_{}.json", transfer_data.transfer_id);
    
    // Create build directory
    fs::create_dir_all("./zisk-tx-proof/build")?;
    
    // Create JSON input with the transaction data
    let input_data = serde_json::json!({
        "transfer_id": transfer_data.transfer_id,
        "block_number": transfer_data.block_number,
        "tx_index": transfer_data.tx_index,
        "from_account": transfer_data.from_account,
        "to_account": transfer_data.to_account,
        "amount": transfer_data.amount
    });
    
    fs::write(&input_file, serde_json::to_string_pretty(&input_data)?)?;
    info!("📝 Created ZisK input file: {}", input_file);
    
    Ok(input_file)
}

/// Build the ZisK program
async fn build_zisk_program() -> Result<()> {
    info!("🔨 Building ZisK program...");
    
    let output = Command::new("cargo-zisk")
        .args(&["build", "--release"])
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

/// Test with ZisK emulator
async fn test_zisk_program(input_file: &str) -> Result<()> {
    info!("🧪 Testing ZisK program with emulator...");
    
    let elf_path = "./zisk-tx-proof/target/riscv64ima-zisk-zkvm-elf/release/zisk-tx-proof";
    
    let output = Command::new("ziskemu")
        .args(&["-e", elf_path, "-i", input_file])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "ZisK emulator test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    info!("✅ ZisK emulator test passed");
    info!("Emulator output: {}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

/// Generate ZisK proof files
async fn generate_zisk_proof_files(input_file: &str) -> Result<String> {
    info!("🔐 Generating ZisK proof...");
    
    let elf_path = "./zisk-tx-proof/target/riscv64ima-zisk-zkvm-elf/release/zisk-tx-proof";
    let proof_dir = format!("./proofs/proof_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs());
    
    // Create proofs directory
    fs::create_dir_all(&proof_dir)?;
    
    // Generate ROM setup
    let setup_output = Command::new("cargo-zisk")
        .args(&["rom-setup", "-e", elf_path])
        .current_dir("./zisk-tx-proof")
        .output()?;
    
    if !setup_output.status.success() {
        warn!("ROM setup warning: {}", String::from_utf8_lossy(&setup_output.stderr));
    }
    
    // Generate the proof
    let proof_output = Command::new("cargo-zisk")
        .args(&[
            "prove",
            "-e", elf_path,
            "-i", input_file,
            "-o", &proof_dir,
            "-a", // Auto-verify
            "-y"  // Yes to all prompts
        ])
        .current_dir("./zisk-tx-proof")
        .output()?;
    
    if !proof_output.status.success() {
        return Err(anyhow::anyhow!(
            "ZisK proof generation failed: {}",
            String::from_utf8_lossy(&proof_output.stderr)
        ));
    }
    
    info!("✅ ZisK proof generated in: {}", proof_dir);
    Ok(proof_dir)
}

/// Parse ZisK proof output
fn parse_zisk_proof_output(proof_dir: &str, transfer_data: &TransactionData) -> Result<TransactionProof> {
    let publics_file = format!("{}/publics.json", proof_dir);
    
    let is_valid = if Path::new(&publics_file).exists() {
        let publics_content = fs::read_to_string(&publics_file)?;
        // In a real implementation, properly parse the JSON to extract validity
        !publics_content.contains("false")
    } else {
        false
    };
    
    let inclusion_proof_hash = compute_inclusion_hash(transfer_data);
    
    Ok(TransactionProof {
        transfer_id: transfer_data.transfer_id,
        block_number: transfer_data.block_number,
        inclusion_proof_hash,
        is_valid,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        proof_path: Some(proof_dir.to_string()),
        proof_type: "zisk".to_string(),
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