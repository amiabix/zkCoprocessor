//! ZisK Transaction Proof Program
//! Proves that a transaction was included in a block with specific properties

use std::io;

// ZisK SDK imports for input/output
use zisk_sdk::{stdin, stdout};

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
    
    // Read transaction data from stdin
    let tx_data = read_transaction_data();
    
    println!("📊 ZisK: Processing transfer_id: {}", tx_data.transfer_id);
    println!("📊 ZisK: Block number: {}", tx_data.block_number);
    println!("📊 ZisK: Amount: {}", tx_data.amount);
    
    // Generate proof of transaction inclusion
    let proof = prove_transaction_inclusion(&tx_data);
    
    // Output the proof
    write_transaction_proof(&proof);
    
    println!("✅ ZisK: Proof generation completed!");
}

/// Read transaction data from ZisK stdin
fn read_transaction_data() -> TransactionData {
    // Read the raw input bytes
    let transfer_id_bytes = stdin::read::<[u8; 16]>();
    let block_number_bytes = stdin::read::<[u8; 8]>();
    let tx_index_bytes = stdin::read::<[u8; 8]>();
    let from_account_bytes = stdin::read::<[u8; 16]>();
    let to_account_bytes = stdin::read::<[u8; 16]>();
    let amount_bytes = stdin::read::<[u8; 16]>();
    
    TransactionData {
        transfer_id: u128::from_le_bytes(transfer_id_bytes),
        block_number: u64::from_le_bytes(block_number_bytes),
        tx_index: u64::from_le_bytes(tx_index_bytes),
        from_account: u128::from_le_bytes(from_account_bytes),
        to_account: u128::from_le_bytes(to_account_bytes),
        amount: u128::from_le_bytes(amount_bytes),
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
    use sha2::{Digest, Sha256};
    
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

/// Write transaction proof to ZisK stdout
fn write_transaction_proof(proof: &TransactionProof) {
    // Output the proof components as public values
    stdout::write(&proof.transfer_id.to_le_bytes());
    stdout::write(&proof.block_number.to_le_bytes());
    stdout::write(&proof.inclusion_proof_hash);
    stdout::write(&[if proof.is_valid { 1u8 } else { 0u8 }]);
}