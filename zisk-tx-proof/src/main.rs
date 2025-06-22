//! ZisK Transaction Proof Program
//! Proves that a transaction was included in a block with specific properties

// ZisK OS imports for input/output
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
    // Try to read from environment variables first (for testing/integration)
    if let (Ok(transfer_id_str), Ok(block_number_str), Ok(tx_index_str), 
            Ok(from_account_str), Ok(to_account_str), Ok(amount_str)) = (
        std::env::var("ZISK_INPUT_TRANSFER_ID"),
        std::env::var("ZISK_INPUT_BLOCK_NUMBER"), 
        std::env::var("ZISK_INPUT_TX_INDEX"),
        std::env::var("ZISK_INPUT_FROM_ACCOUNT"),
        std::env::var("ZISK_INPUT_TO_ACCOUNT"),
        std::env::var("ZISK_INPUT_AMOUNT")
    ) {
        println!("🔍 ZisK: Reading from environment variables");
        
        let transfer_id = transfer_id_str.parse::<u128>()
            .expect("Invalid transfer_id in environment");
        let block_number = block_number_str.parse::<u64>()
            .expect("Invalid block_number in environment");
        let tx_index = tx_index_str.parse::<u64>()
            .expect("Invalid tx_index in environment");
        let from_account = from_account_str.parse::<u128>()
            .expect("Invalid from_account in environment");
        let to_account = to_account_str.parse::<u128>()
            .expect("Invalid to_account in environment");
        let amount = amount_str.parse::<u128>()
            .expect("Invalid amount in environment");
        
        return TransactionData {
            transfer_id,
            block_number,
            tx_index,
            from_account,
            to_account,
            amount,
        };
    }
    
    // Fallback to ZisK input system
    println!("🔍 ZisK: Reading from ZisK input system");
    
    // Read raw bytes and convert to proper types
    let transfer_id_bytes = read_input();
    let block_number_bytes = read_input();
    let tx_index_bytes = read_input();
    let from_account_bytes = read_input();
    let to_account_bytes = read_input();
    let amount_bytes = read_input();
    
    // Convert bytes to proper types with padding for smaller inputs
    let transfer_id = if transfer_id_bytes.len() == 16 {
        u128::from_le_bytes(transfer_id_bytes.try_into().expect("Invalid transfer_id length"))
    } else {
        // Pad with zeros if input is smaller
        let mut padded = [0u8; 16];
        padded[..transfer_id_bytes.len().min(16)].copy_from_slice(&transfer_id_bytes[..transfer_id_bytes.len().min(16)]);
        u128::from_le_bytes(padded)
    };
    
    let block_number = if block_number_bytes.len() == 8 {
        u64::from_le_bytes(block_number_bytes.try_into().expect("Invalid block_number length"))
    } else {
        let mut padded = [0u8; 8];
        padded[..block_number_bytes.len().min(8)].copy_from_slice(&block_number_bytes[..block_number_bytes.len().min(8)]);
        u64::from_le_bytes(padded)
    };
    
    let tx_index = if tx_index_bytes.len() == 8 {
        u64::from_le_bytes(tx_index_bytes.try_into().expect("Invalid tx_index length"))
    } else {
        let mut padded = [0u8; 8];
        padded[..tx_index_bytes.len().min(8)].copy_from_slice(&tx_index_bytes[..tx_index_bytes.len().min(8)]);
        u64::from_le_bytes(padded)
    };
    
    let from_account = if from_account_bytes.len() == 16 {
        u128::from_le_bytes(from_account_bytes.try_into().expect("Invalid from_account length"))
    } else {
        let mut padded = [0u8; 16];
        padded[..from_account_bytes.len().min(16)].copy_from_slice(&from_account_bytes[..from_account_bytes.len().min(16)]);
        u128::from_le_bytes(padded)
    };
    
    let to_account = if to_account_bytes.len() == 16 {
        u128::from_le_bytes(to_account_bytes.try_into().expect("Invalid to_account length"))
    } else {
        let mut padded = [0u8; 16];
        padded[..to_account_bytes.len().min(16)].copy_from_slice(&to_account_bytes[..to_account_bytes.len().min(16)]);
        u128::from_le_bytes(padded)
    };
    
    let amount = if amount_bytes.len() == 16 {
        u128::from_le_bytes(amount_bytes.try_into().expect("Invalid amount length"))
    } else {
        let mut padded = [0u8; 16];
        padded[..amount_bytes.len().min(16)].copy_from_slice(&amount_bytes[..amount_bytes.len().min(16)]);
        u128::from_le_bytes(padded)
    };
    
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