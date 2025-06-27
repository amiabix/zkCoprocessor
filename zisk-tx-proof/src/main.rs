
// ZisK Transaction Verification Circuit
// This circuit proves that a transfer_id meets certain validity constraints

#![no_main]
ziskos::entrypoint!(main);

use ziskos::{read_input, set_output};
use std::convert::TryInto;

fn main() {
    // Read the input data as a byte array from ziskos
    let input: Vec<u8> = read_input();
    
    // Convert input bytes to u128 transfer_id
    // ZisK input.bin contains our transfer_id as 16 bytes (u128)
    if input.len() != 16 {
        panic!("Invalid input length: expected 16 bytes for u128");
    }
    
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
    for i in 0..2 {
        let chunk = u32::from_le_bytes([
            result_bytes[i*4], 
            result_bytes[i*4+1], 
            result_bytes[i*4+2], 
            result_bytes[i*4+3]
        ]);
        set_output(i, chunk);
    }
    
    // Output success flag
    set_output(2, 1); // 1 means success
}

fn compute_verification_hash(transfer_id: u128) -> u64 {
    // Simple hash computation for demonstration
    // In production, you'd use a proper cryptographic hash
    let mut hash: u64 = 0;
    
    // Hash the transfer_id in chunks
    let bytes = transfer_id.to_le_bytes();
    for i in 0..2 {
        let chunk = u64::from_le_bytes([
            bytes[i*8], bytes[i*8+1], bytes[i*8+2], bytes[i*8+3],
            bytes[i*8+4], bytes[i*8+5], bytes[i*8+6], bytes[i*8+7]
        ]);
        hash = hash.wrapping_add(chunk);
    }
    
    // Ensure non-zero result
    if hash == 0 { hash = 1; }
    
    hash
}
