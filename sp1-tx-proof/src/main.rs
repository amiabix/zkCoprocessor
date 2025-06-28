#![no_main]
sp1_lib::entrypoint!(main);

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub transfer_id: u128,
    pub block_number: u64,
    pub from_account: u128,
    pub to_account: u128,
    pub amount: u128,
}

pub fn main() {
    let input_bytes = sp1_lib::io::read_vec();
    let tx_data: TransactionData = bincode::deserialize(&input_bytes)
        .expect("Failed to deserialize");
    
    // Same validation logic as ZisK circuit
    let is_valid = tx_data.transfer_id > 0 
        && tx_data.transfer_id >= 19000000000000 
        && tx_data.transfer_id < 20000000000000
        && tx_data.amount > 0;
    
    sp1_lib::io::commit(&(is_valid as u32));
    sp1_lib::io::commit(&(tx_data.transfer_id as u64));
}
