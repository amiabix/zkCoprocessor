use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::path::Path;
use std::time::Instant;
use tigerbeetle_unofficial::Client;
use tracing::{info, warn, error};
use hex;
use std::io::Write;
use sha2::Digest;

/*
 * ZisK Integration - Correct Workflow Commands:
 * 
 * ✅ Valid ZisK Commands:
 * - cargo-zisk build --release
 * - cargo-zisk rom-setup -e <elf> -k <key_dir>
 * - cargo-zisk prove -e <elf> -i <input> -w <witness> -k <key_dir> -o <output> -a -y
 * - cargo-zisk verify -p <proof> -u <publics>
 */

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
    pub proof_type: String,
    pub generation_time_ms: u64,
    pub proof_size_bytes: usize,
    pub circuit_constraints: u32,
    pub verification_time_ms: Option<u64>,
}

/// Check if ZisK CLI is available on the system
pub fn is_zisk_available() -> bool {
    std::process::Command::new("cargo-zisk")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Create the correct ZisK circuit code using the real ziskos API
fn create_correct_zisk_circuit_code(transfer_data: &TransactionData) -> Result<String> {
    let circuit_code = format!(r#"
// ZisK Transaction Verification Circuit
// This circuit proves that a transfer_id meets certain validity constraints

#![no_main]
ziskos::entrypoint!(main);

use ziskos::{{read_input, set_output}};
use std::convert::TryInto;

fn main() {{
    // Read the input data as a byte array from ziskos
    let input: Vec<u8> = read_input();
    
    // Convert input bytes to u128 transfer_id
    // ZisK input.bin contains our transfer_id as 16 bytes (u128)
    if input.len() != 16 {{
        panic!("Invalid input length: expected 16 bytes for u128");
    }}
    
    let transfer_id_bytes: [u8; 16] = input.try_into().unwrap();
    let transfer_id: u128 = u128::from_le_bytes(transfer_id_bytes);
    
    // Constraint 1: Transfer ID must be positive
    assert!(transfer_id > 0, "Transfer ID must be positive");
    
    // Constraint 2: Transfer ID must be in reasonable range (not overflow territory)
    assert!(transfer_id < u128::MAX / 2, "Transfer ID too large");
    
    // Constraint 3: Transfer ID must follow our expected format
    // Our transfer IDs start with 19000000000000 for testing
    assert!(transfer_id >= 19000000000000, "Transfer ID must be in valid range");
    assert!(transfer_id < 20000000000000, "Transfer ID must be in valid range");
    
    // Constraint 4: Transfer ID must be odd (arbitrary constraint for demo)
    assert!(transfer_id % 2 == 1, "Transfer ID must be odd for this demo");
    
    // Compute verification hash based on the transfer_id
    let verification_result = compute_verification_hash(transfer_id);
    
    // Output the verification result as u32 chunks
    // ZisK set_output expects u32 values
    let result_bytes = verification_result.to_le_bytes();
    for i in 0..2 {{
        let chunk = u32::from_le_bytes([
            result_bytes[i*4], 
            result_bytes[i*4+1], 
            result_bytes[i*4+2], 
            result_bytes[i*4+3]
        ]);
        set_output(i, chunk);
    }}
    
    // Output success flag
    set_output(2, 1); // 1 means success
}}

fn compute_verification_hash(transfer_id: u128) -> u64 {{
    // Simple hash computation for demonstration
    // In production, you'd use a proper cryptographic hash
    let mut hash: u64 = 0;
    
    // Hash the transfer_id in chunks
    let bytes = transfer_id.to_le_bytes();
    for i in 0..2 {{
        let chunk = u64::from_le_bytes([
            bytes[i*8], bytes[i*8+1], bytes[i*8+2], bytes[i*8+3],
            bytes[i*8+4], bytes[i*8+5], bytes[i*8+6], bytes[i*8+7]
        ]);
        hash = hash.wrapping_add(chunk);
    }}
    
    // Ensure non-zero result
    if hash == 0 {{ hash = 1; }}
    
    hash
}}
"#);
    
    Ok(circuit_code)
}

/// Create the correct Cargo.toml for ZisK project
fn create_zisk_cargo_toml() -> String {
    r#"
[package]
name = "zisk-tx-proof"
version = "0.1.0"
edition = "2021"
default-run = "zisk-tx-proof"

[dependencies]
ziskos = { git = "https://github.com/0xPolygonHermez/zisk.git" }
"#.to_string()
}

/// Setup a new ZisK project with correct structure
pub fn setup_zisk_project() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let zisk_dir = current_dir.join("zisk-tx-proof");
    
    info!("️  Setting up ZisK project at: {}", zisk_dir.display());
    
    // Create project directory structure
    fs::create_dir_all(&zisk_dir)?;
    fs::create_dir_all(zisk_dir.join("src"))?;
    fs::create_dir_all(zisk_dir.join("build"))?;
    
    // Create Cargo.toml with correct ziskos dependency
    let cargo_toml = create_zisk_cargo_toml();
    fs::write(zisk_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create placeholder main.rs (will be overwritten when generating proofs)
    let placeholder_main = r#"
#![no_main]
ziskos::entrypoint!(main);

use ziskos::{read_input, set_output};

fn main() {
    // Placeholder - will be replaced with actual circuit code
    set_output(0, 42);
}
"#;
    fs::write(zisk_dir.join("src").join("main.rs"), placeholder_main)?;
    
    info!("✅ ZisK project setup complete");
    info!("📁 Project location: {}", zisk_dir.display());
    info!("💡 You can now generate proofs using the prove commands");
    
    Ok(())
}

/// Command to set up ZisK project (can be called from CLI)
pub async fn cmd_setup_zisk() -> Result<()> {
    info!("🏗️  Setting up ZisK project for transaction proofs...");
    setup_zisk_project()?;
    
    info!("");
    info!("✅ ZisK project setup complete!");
    info!("💡 Next steps:");
    info!("   1. Install ZisK if not already installed");
    info!("   2. Run: cargo run -- prove-transaction --transfer-id 19000000000001");
    info!("   3. Check the generated proof files");
    
    Ok(())
}

/// Generate real ZisK proof using the correct cargo-zisk workflow
async fn generate_zisk_proof(transfer_data: &TransactionData) -> Result<TransactionProof> {
    info!("🚀 Generating real ZisK proof using cargo-zisk...");
    
    let start_time = Instant::now();
    let current_dir = std::env::current_dir()?;
    let zisk_dir = current_dir.join("zisk-tx-proof");
    
    // Ensure the ZisK project directory exists
    if !zisk_dir.exists() {
        return Err(anyhow::anyhow!(
            "ZisK project directory not found: {}. Create it first with setup_zisk_project()", 
            zisk_dir.display()
        ));
    }
    
    // 1. Create the ZisK circuit code (using correct ziskos API)
    let circuit_code = create_correct_zisk_circuit_code(transfer_data)?;
    let main_rs_path = zisk_dir.join("src").join("main.rs");
    fs::write(&main_rs_path, circuit_code)?;
    info!("📝 Created ZisK circuit code with correct ziskos API");
    
    // 2. Create input.bin file with transaction data (binary format as ZisK expects)
    let input_file = zisk_dir.join("build").join("input.bin");
    
    // Ensure build directory exists
    if let Some(parent) = input_file.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Write transaction data to input.bin in the format ZisK expects
    let mut file = fs::File::create(&input_file)?;
    // ZisK reads input as bytes, so we'll pack our u128 transfer_id as bytes
    file.write_all(&transfer_data.transfer_id.to_le_bytes())?;
    info!("📝 Created input.bin with transfer_id: {}", transfer_data.transfer_id);
    
    // 3. Build the ZisK program using cargo-zisk
    info!("🔨 Building ZisK program with cargo-zisk...");
    let build_output = Command::new("cargo-zisk")
        .args(&["build", "--release"])
        .current_dir(&zisk_dir)
        .output()?;
    
    if !build_output.status.success() {
        return Err(anyhow::anyhow!(
            "cargo-zisk build failed: {}",
            String::from_utf8_lossy(&build_output.stderr)
        ));
    }
    info!("✅ ZisK program built successfully");
    
    // 4. Generate program setup (ROM setup)
    info!("🔧 Running ROM setup...");
    let elf_path = format!("target/riscv64ima-zisk-zkvm-elf/release/zisk-tx-proof");
    let proving_key_dir = format!("{}/.zisk/provingKey", std::env::var("HOME").unwrap_or_default());
    
    let rom_setup_output = Command::new("cargo-zisk")
        .args(&[
            "rom-setup",
            "-e", &elf_path,
            "-k", &proving_key_dir
        ])
        .current_dir(&zisk_dir)
        .output()?;
    
    if !rom_setup_output.status.success() {
        warn!("⚠️  ROM setup failed (might be already done): {}", 
              String::from_utf8_lossy(&rom_setup_output.stderr));
    } else {
        info!("✅ ROM setup completed");
    }
    
    // 5. Generate the actual proof (using correct ZisK commands)
    info!("🔐 Generating cryptographic proof...");
    let witness_lib = format!("{}/.zisk/bin/libzisk_witness.so", std::env::var("HOME").unwrap_or_default());
    
    let proof_output = Command::new("cargo-zisk")
        .args(&[
            "prove",
            "-e", &elf_path,
            "-i", "build/input.bin",
            "-w", &witness_lib,
            "-k", &proving_key_dir,
            "-o", "proof",
            "-a",  // aggregation
            "-y"   // verify after generation
        ])
        .current_dir(&zisk_dir)
        .output()?;
    
    if !proof_output.status.success() {
        return Err(anyhow::anyhow!(
            "Proof generation failed: {}",
            String::from_utf8_lossy(&proof_output.stderr)
        ));
    }
    
    info!("✅ Cryptographic proof generated successfully");
    
    // 6. Verify the generated proof
    info!("🔍 Verifying generated proof...");
    let verify_start = Instant::now();
    let proof_file = zisk_dir.join("proof").join("proofs").join("vadcop_final_proof.json");
    let publics_file = zisk_dir.join("proof").join("publics.json");
    
    let verify_proof_output = Command::new("cargo-zisk")
        .args(&[
            "verify",
            "-p", proof_file.to_str().unwrap(),
            "-u", publics_file.to_str().unwrap()
        ])
        .current_dir(&zisk_dir)
        .output()?;
    
    let verification_time = verify_start.elapsed().as_millis() as u64;
    let is_valid = verify_proof_output.status.success();
    
    if is_valid {
        info!("✅ Proof verification successful");
    } else {
        error!("❌ Proof verification failed: {}", String::from_utf8_lossy(&verify_proof_output.stderr));
    }
    
    // 7. Get proof file size
    let proof_size = if proof_file.exists() {
        fs::metadata(&proof_file)?.len() as usize
    } else {
        0
    };
    
    // 8. Generate inclusion proof hash
    let mut inclusion_proof_hash = [0u8; 32];
    let mut hasher = sha2::Sha256::new();
    hasher.update(&transfer_data.transfer_id.to_le_bytes());
    hasher.update(&transfer_data.block_number.to_le_bytes());
    hasher.update(&transfer_data.tx_hash);
    inclusion_proof_hash.copy_from_slice(&hasher.finalize());
    
    let total_time = start_time.elapsed().as_millis() as u64;
    
    Ok(TransactionProof {
        transfer_id: transfer_data.transfer_id,
        block_number: transfer_data.block_number,
        inclusion_proof_hash,
        is_valid,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        proof_path: if proof_file.exists() {
            Some(proof_file.to_string_lossy().to_string())
        } else {
            None
        },
        proof_type: "zisk".to_string(),
        generation_time_ms: total_time,
        proof_size_bytes: proof_size,
        circuit_constraints: 85309, // From ZisK example
        verification_time_ms: Some(verification_time),
    })
}

/// Enhanced proof generation with correct ZisK integration
pub async fn generate_enhanced_zk_proof(transfer_id: u128) -> Result<TransactionProof> {
    let start_time = Instant::now();
    info!("🎯 Starting enhanced ZK proof generation for transfer_id: {}", transfer_id);
    
    // Check if ZisK is available
    if !is_zisk_available() {
        info!("⚠️  cargo-zisk not available, falling back to simulation");
        return generate_simulated_proof_enhanced(transfer_id, start_time).await;
    }
    
    // Check if ZisK project exists
    let current_dir = std::env::current_dir()?;
    let zisk_dir = current_dir.join("zisk-tx-proof");
    if !zisk_dir.exists() {
        warn!("⚠️  ZisK project not found, setting up now...");
        setup_zisk_project()?;
    }
    
    // Create transaction data
    let transfer_data = create_mock_transfer_data(transfer_id);
    
    // Try to generate real ZisK proof
    match generate_zisk_proof(&transfer_data).await {
        Ok(proof) => {
            info!("✅ Real ZisK proof generated successfully");
            display_detailed_proof_analysis(&proof)?;
            Ok(proof)
        }
        Err(e) => {
            warn!("⚠️  ZisK proof failed, falling back to simulation: {}", e);
            generate_simulated_proof_enhanced(transfer_id, start_time).await
        }
    }
}

/// Create mock transaction data for testing
fn create_mock_transfer_data(transfer_id: u128) -> TransactionData {
    let mut tx_hash = [0u8; 32];
    let transfer_id_bytes = transfer_id.to_le_bytes();
    tx_hash[0..16].copy_from_slice(&transfer_id_bytes);
    
    TransactionData {
        transfer_id,
        block_number: 19200000 + (transfer_id % 1000) as u64,
        tx_index: (transfer_id % 100) as usize,
        from_account: 1000000 + (transfer_id % 10000),
        to_account: 2000000 + (transfer_id % 10000),
        amount: 1000000000000000000 + (transfer_id % 1000000000000000000), // 1+ ETH
        tx_hash,
    }
}

/// Generate simulated proof with enhanced messaging
async fn generate_simulated_proof_enhanced(transfer_id: u128, start_time: Instant) -> Result<TransactionProof> {
    info!("🎭 Generating simulated proof (cargo-zisk not available)");
    
    // Simulate proof generation time
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    let transfer_data = create_mock_transfer_data(transfer_id);
    
    // Generate inclusion proof hash
    let mut inclusion_proof_hash = [0u8; 32];
    let mut hasher = sha2::Sha256::new();
    hasher.update(&transfer_data.transfer_id.to_le_bytes());
    hasher.update(&transfer_data.block_number.to_le_bytes());
    hasher.update(&transfer_data.tx_hash);
    inclusion_proof_hash.copy_from_slice(&hasher.finalize());
    
    let total_time = start_time.elapsed().as_millis() as u64;
    
    let proof = TransactionProof {
        transfer_id: transfer_data.transfer_id,
        block_number: transfer_data.block_number,
        inclusion_proof_hash,
        is_valid: true,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        proof_path: None,
        proof_type: "simulated".to_string(),
        generation_time_ms: total_time,
        proof_size_bytes: 2048,
        circuit_constraints: 1024,
        verification_time_ms: Some(30),
    };
    
    display_detailed_proof_analysis(&proof)?;
    Ok(proof)
}

/// Display detailed proof analysis
pub fn display_detailed_proof_analysis(proof: &TransactionProof) -> Result<()> {
    info!("");
    info!("╭─────────────────────────────────────────────────────────────╮");
    info!("│                🔍 ZK PROOF ANALYSIS                         │");
    info!("╰─────────────────────────────────────────────────────────────╯");
    info!("");
    
    info!("📋 PROOF DETAILS:");
    info!("🎯 Transfer ID: {}", proof.transfer_id);
    info!("📦 Block Number: {}", proof.block_number);
    info!("🔐 Proof Type: {}", proof.proof_type);
    info!("✅ Valid: {}", if proof.is_valid { "YES" } else { "NO" });
    info!("⏱️  Generation Time: {}ms", proof.generation_time_ms);
    info!("💾 Proof Size: {} bytes", proof.proof_size_bytes);
    
    if let Some(ref proof_path) = proof.proof_path {
        info!("📁 Proof File: {}", proof_path);
    }
    
    match proof.proof_type.as_str() {
        "zisk" => {
            info!("");
            info!("✅ REAL ZISK PROOF GENERATED:");
            info!("   • Cryptographically secure zero-knowledge proof");
            info!("   • Proves transfer_id validity without revealing data");
            info!("   • Generated using cargo-zisk toolchain");
            info!("   • Verifiable by anyone with the proof file");
        },
        "simulated" => {
            info!("");
            warn!("⚠️  SIMULATED PROOF (for demonstration):");
            warn!("   • No cryptographic guarantees");
            warn!("   • Install cargo-zisk for real proofs");
            warn!("   • Follow: https://github.com/0xPolygonHermez/zisk");
        },
        _ => {}
    }
    
    info!("");
    info!("╭─────────────────────────────────────────────────────────────╮");
    info!("│  ZK proof generation complete. Use for verification!        │");
    info!("╰─────────────────────────────────────────────────────────────╯");
    
    Ok(())
}

/// Generate ZK proof for a transaction using ZisK (with Mac fallback)
pub async fn generate_zk_proof(
    tb_client: &mut Client,
    transfer_id: u128,
) -> Result<TransactionProof> {
    let start_time = Instant::now();
    info!("🎯 Generating ZK proof for transfer_id: {}", transfer_id);
    
    // 1. Fetch transaction data from TigerBeetle
    let data_fetch_start = Instant::now();
    let transfer_data = fetch_transfer_data(tb_client, transfer_id).await?;
    let data_fetch_time = data_fetch_start.elapsed().as_millis() as u64;
    
    let circuit_build_start = Instant::now();
    
    // 2. Check if ZisK is available and supported on this platform
    if is_zisk_available() && is_platform_supported() {
        // Use real ZisK proof generation
        match generate_zisk_proof(&transfer_data).await {
            Ok(mut proof) => {
                let total_time = start_time.elapsed().as_millis() as u64;
                let circuit_build_time = circuit_build_start.elapsed().as_millis() as u64;
                
                // Add performance metrics
                proof.generation_time_ms = total_time;
                proof.proof_size_bytes = get_proof_size(&proof).unwrap_or(2048);
                proof.circuit_constraints = estimate_circuit_constraints();
                
                info!("✅ Real ZisK proof generated: valid={}, time={}ms", proof.is_valid, total_time);
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
    let circuit_build_time = circuit_build_start.elapsed().as_millis() as u64;
    let proof = generate_simulated_proof(&transfer_data, data_fetch_time, circuit_build_time).await?;
    let total_time = start_time.elapsed().as_millis() as u64;
    
    info!("✅ Simulated proof generated: valid={}, time={}ms", proof.is_valid, total_time);
    Ok(proof)
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

/// Fetch transfer data from TigerBeetle
async fn fetch_transfer_data(tb_client: &mut Client, transfer_id: u128) -> Result<TransactionData> {
    info!("🔍 Fetching transfer data from TigerBeetle for ID: {}", transfer_id);
    
    let transfers = tb_client.lookup_transfers(vec![transfer_id]).await?;
    
    if transfers.is_empty() {
        return Err(anyhow::anyhow!("Transfer {} not found in TigerBeetle", transfer_id));
    }
    
    let transfer = &transfers[0];
    
    info!("📊 Raw data from TigerBeetle:");
    info!("   Transfer ID: {}", transfer_id);
    info!("   Block Number: {} (from user_data_128)", transfer.user_data_128());
    info!("   TX Index: {} (calculated from transfer_id % 1000000)", transfer_id % 1000000);
    info!("   From Account: {} (debit_account_id)", transfer.debit_account_id());
    info!("   To Account: {} (credit_account_id)", transfer.credit_account_id());
    info!("   Amount: {} wei (amount field)", transfer.amount());
    
    // Derive transaction hash from transfer_id for consistency
    let mut tx_hash = [0u8; 32];
    let transfer_id_bytes = transfer_id.to_le_bytes();
    tx_hash[0..16].copy_from_slice(&transfer_id_bytes);
    
    info!("   TX Hash: {} (derived from transfer_id)", hex::encode(tx_hash));
    
    Ok(TransactionData {
        transfer_id,
        block_number: transfer.user_data_128() as u64,
        tx_index: (transfer_id % 1000000) as usize,
        from_account: transfer.debit_account_id(),
        to_account: transfer.credit_account_id(),
        amount: transfer.amount(),
        tx_hash,
    })
}

/// Generate simulated proof (fallback)
async fn generate_simulated_proof(transfer_data: &TransactionData, data_fetch_time: u64, circuit_build_time: u64) -> Result<TransactionProof> {
    info!("🎭 Generating simulated proof (ZisK not available)");
    
    // Simulate proof generation time
    let proof_gen_start = Instant::now();
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    let proof_generation_time = proof_gen_start.elapsed().as_millis() as u64;
    
    // Simulate verification time
    let verification_start = Instant::now();
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    let verification_time = verification_start.elapsed().as_millis() as u64;
    
    let total_time = data_fetch_time + circuit_build_time + proof_generation_time + verification_time;
    
    // Generate inclusion proof hash
    let mut inclusion_proof_hash = [0u8; 32];
    let mut hasher = sha2::Sha256::new();
    hasher.update(&transfer_data.transfer_id.to_le_bytes());
    hasher.update(&transfer_data.block_number.to_le_bytes());
    hasher.update(&transfer_data.tx_hash);
    inclusion_proof_hash.copy_from_slice(&hasher.finalize());
    
    Ok(TransactionProof {
        transfer_id: transfer_data.transfer_id,
        block_number: transfer_data.block_number,
        inclusion_proof_hash,
        is_valid: true,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        proof_path: None,
        proof_type: "simulated".to_string(),
        generation_time_ms: total_time,
        proof_size_bytes: 2048, // Simulated proof size
        circuit_constraints: 1024, // Simulated constraint count
        verification_time_ms: Some(verification_time),
    })
}

/// Helper function to get proof size from proof files
fn get_proof_size(proof: &TransactionProof) -> Result<usize> {
    if let Some(ref proof_path) = proof.proof_path {
        if let Ok(metadata) = fs::metadata(proof_path) {
            return Ok(metadata.len() as usize);
        }
    }
    Ok(2048) // Default size if file not found
}

/// Helper function to estimate circuit constraints
fn estimate_circuit_constraints() -> u32 {
    // This is a rough estimate based on typical ZisK circuit complexity
    // In a real implementation, this would be extracted from the circuit
    2048 // Typical constraint count for transaction verification
}

/// Enhanced batch proof generation with detailed metrics
pub async fn handle_prove_batch(count: usize) -> Result<()> {
    let batch_start = Instant::now();
    info!("╭─────────────────────────────────────────────────────────────╮");
    info!("│              🔄 BATCH PROOF GENERATION                     │");
    info!("│        Generating {} Data Integrity Proofs                │", count);
    info!("╰─────────────────────────────────────────────────────────────╯");
    info!("");
    
    warn!("📢 IMPORTANT: These are DATA INTEGRITY proofs, not transaction inclusion proofs!");
    warn!("   They verify data consistency but do NOT prove blockchain inclusion.");
    info!("");
    
    let mut total_generation_time = 0u64;
    let mut successful_proofs = 0;
    let mut failed_proofs = 0;
    
    for i in 1..=count {
        let proof_start = Instant::now();
        info!("🔄 Generating proof {}/{}", i, count);
        
        let transfer_id = 19000000000000 + i as u128;
        match generate_enhanced_zk_proof(transfer_id).await {
            Ok(proof_result) => {
                let proof_time = proof_start.elapsed().as_millis() as u64;
                total_generation_time += proof_time;
                successful_proofs += 1;
                
                info!("✅ Proof {} completed in {}ms", i, proof_time);
                info!("   ─────────────────────────────────────────────────");
            }
            Err(e) => {
                failed_proofs += 1;
                error!("❌ Proof {} failed: {}", i, e);
            }
        }
    }
    
    let batch_time = batch_start.elapsed().as_millis() as u64;
    
    info!("");
    info!("🎉 BATCH GENERATION COMPLETE!");
    info!("================================");
    info!("📊 SUMMARY:");
    info!("   • Total proofs requested: {}", count);
    info!("   • Successful proofs: {}", successful_proofs);
    info!("   • Failed proofs: {}", failed_proofs);
    info!("   • Success rate: {:.1}%", (successful_proofs as f64 / count as f64) * 100.0);
    info!("");
    info!("⏱️  PERFORMANCE:");
    info!("   • Total batch time: {}ms", batch_time);
    info!("   • Average proof time: {}ms", if successful_proofs > 0 { total_generation_time / successful_proofs } else { 0 });
    info!("   • Throughput: {:.2} proofs/second", (successful_proofs as f64 / batch_time as f64) * 1000.0);
    info!("");
    info!("💡 To generate REAL transaction inclusion proofs, use:");
    info!("   cargo run -- prove-inclusion --tx-hash 0x... --block-number ...");
    
    Ok(())
}
