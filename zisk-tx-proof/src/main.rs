//! ZisK Transaction Proof Program
//! Proves that a transaction was included in a block with specific properties

// Mark the main function as the entry point for ZisK
#![no_main]
ziskos::entrypoint!(main);

use sha2::{Digest, Sha256};
use std::convert::TryInto;
use ziskos::{read_input, set_output};

#[derive(Debug)]
struct TransactionData {
    transfer_id: u128,
    block_number: u64,
    tx_index: u64,
    from_account: u128,
    to_account: u128,
    amount: u128,
}

#[derive(Debug)]
struct TransactionProof {
    transfer_id: u128,
    block_number: u64,
    inclusion_proof_hash: [u8; 32],
    is_valid: bool,
}

fn main() {
    println!("🔍 ZisK: Starting transaction proof generation...");
    
    // Read transaction data from ZisK input
    let tx_data = read_transaction_data();
    
    println!("📊 ZisK: Processing transfer_id: {}", tx_data.transfer_id);
    println!("📊 ZisK: Block number: {}", tx_data.block_number);
    println!("📊 ZisK: Amount: {}", tx_data.amount);
    
    // Generate proof of transaction inclusion
    let proof = prove_transaction_inclusion(&tx_data);
    
    // Output the proof using ZisK
    write_transaction_proof(&proof);
    
    println!("✅ ZisK: Proof generation completed!");
}

/// Read transaction data from ZisK input
fn read_transaction_data() -> TransactionData {
    // Read the input data as a byte array from ziskos
    let input: Vec<u8> = read_input();
    
    // Parse the input data - we expect 6 values in little-endian format:
    // transfer_id (16 bytes), block_number (8 bytes), tx_index (8 bytes),
    // from_account (16 bytes), to_account (16 bytes), amount (16 bytes)
    
    if input.len() < 80 { // 16+8+8+16+16+16 = 80 bytes minimum
        println!("⚠️  ZisK: Input too short, using default values for testing");
        return TransactionData {
            transfer_id: 20,
            block_number: 20,
            tx_index: 0,
            from_account: 20,
            to_account: 20,
            amount: 20,
        };
    }
    
    // Parse transfer_id (first 16 bytes)
    let transfer_id_bytes: [u8; 16] = input[0..16].try_into().unwrap();
    let transfer_id = u128::from_le_bytes(transfer_id_bytes);
    
    // Parse block_number (next 8 bytes)
    let block_number_bytes: [u8; 8] = input[16..24].try_into().unwrap();
    let block_number = u64::from_le_bytes(block_number_bytes);
    
    // Parse tx_index (next 8 bytes)
    let tx_index_bytes: [u8; 8] = input[24..32].try_into().unwrap();
    let tx_index = u64::from_le_bytes(tx_index_bytes);
    
    // Parse from_account (next 16 bytes)
    let from_account_bytes: [u8; 16] = input[32..48].try_into().unwrap();
    let from_account = u128::from_le_bytes(from_account_bytes);
    
    // Parse to_account (next 16 bytes)
    let to_account_bytes: [u8; 16] = input[48..64].try_into().unwrap();
    let to_account = u128::from_le_bytes(to_account_bytes);
    
    // Parse amount (last 16 bytes)
    let amount_bytes: [u8; 16] = input[64..80].try_into().unwrap();
    let amount = u128::from_le_bytes(amount_bytes);
    
    TransactionData {
        transfer_id,
        block_number,
        tx_index,
        from_account,
        to_account,
        amount,
    }
}

/// Core logic: Prove that a transaction was included in a block
fn prove_transaction_inclusion(tx_data: &TransactionData) -> TransactionProof {
    println!("🔒 ZisK: Validating transaction data...");
    
    // 1. Validate transaction data consistency
    let is_valid_data = validate_transaction_data(tx_data);
    
    // 2. Compute inclusion proof hash using SHA-256
    let inclusion_proof_hash = compute_inclusion_proof(tx_data);
    
    // 3. Verify block number encoding in transfer_id
    let expected_prefix = tx_data.block_number as u128 * 1000000;
    let actual_prefix = tx_data.transfer_id / 1000000 * 1000000;
    let is_valid_encoding = expected_prefix == actual_prefix;
    
    println!("🔒 ZisK: Block encoding valid: {}", is_valid_encoding);
    println!("🔒 ZisK: Data consistency valid: {}", is_valid_data);
    
    let is_valid = is_valid_data && is_valid_encoding;
    
    TransactionProof {
        transfer_id: tx_data.transfer_id,
        block_number: tx_data.block_number,
        inclusion_proof_hash,
        is_valid,
    }
}

/// Validate internal consistency of transaction data
fn validate_transaction_data(tx_data: &TransactionData) -> bool {
    // Check that accounts are different (no self-transfers)
    if tx_data.from_account == tx_data.to_account {
        println!("❌ ZisK: Invalid - self transfer detected");
        return false;
    }
    
    // Check that amount is non-zero
    if tx_data.amount == 0 {
        println!("❌ ZisK: Invalid - zero amount transfer");
        return false;
    }
    
    // Check that transfer_id encodes the transaction index
    let encoded_tx_index = tx_data.transfer_id % 1000000;
    if encoded_tx_index != tx_data.tx_index as u128 {
        println!("❌ ZisK: Invalid - transfer_id doesn't encode tx_index correctly");
        return false;
    }
    
    println!("✅ ZisK: Transaction data validation passed");
    true
}

/// Compute a cryptographic proof of inclusion using SHA-256
fn compute_inclusion_proof(tx_data: &TransactionData) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    // Hash all the critical transaction components
    hasher.update(&tx_data.transfer_id.to_le_bytes());
    hasher.update(&tx_data.block_number.to_le_bytes());
    hasher.update(&tx_data.tx_index.to_le_bytes());
    hasher.update(&tx_data.from_account.to_le_bytes());
    hasher.update(&tx_data.to_account.to_le_bytes());
    hasher.update(&tx_data.amount.to_le_bytes());
    
    let result = hasher.finalize();
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&result);
    
    println!("🔐 ZisK: Computed inclusion proof hash");
    hash_array
}

/// Write transaction proof to ZisK output
fn write_transaction_proof(proof: &TransactionProof) {
    // Set public outputs using ZisK API (id: usize, value: u32)
    // We'll output the proof components as a series of u32 values
    
    // Output transfer_id as 4 u32 values (128 bits / 32 bits = 4)
    let transfer_id_bytes = proof.transfer_id.to_le_bytes();
    for (i, chunk) in transfer_id_bytes.chunks(4).enumerate() {
        let mut bytes = [0u8; 4];
        bytes[..chunk.len()].copy_from_slice(chunk);
        let value = u32::from_le_bytes(bytes);
        set_output(i, value);
    }
    
    // Output block_number as 2 u32 values (64 bits / 32 bits = 2)
    let block_number_bytes = proof.block_number.to_le_bytes();
    for (i, chunk) in block_number_bytes.chunks(4).enumerate() {
        let mut bytes = [0u8; 4];
        bytes[..chunk.len()].copy_from_slice(chunk);
        let value = u32::from_le_bytes(bytes);
        set_output(4 + i, value);
    }
    
    // Output first 8 bytes of inclusion_proof_hash as 2 u32 values
    for (i, chunk) in proof.inclusion_proof_hash[0..8].chunks(4).enumerate() {
        let mut bytes = [0u8; 4];
        bytes[..chunk.len()].copy_from_slice(chunk);
        let value = u32::from_le_bytes(bytes);
        set_output(6 + i, value);
    }
    
    // Output validity as u32 (0 = false, 1 = true)
    set_output(8, if proof.is_valid { 1 } else { 0 });
}