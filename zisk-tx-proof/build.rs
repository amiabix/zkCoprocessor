use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

// Define constants for the directory and input file name
const OUTPUT_DIR: &str = "build/";
const FILE_NAME: &str = "input.bin";

fn main() -> io::Result<()> {
    // Ensure the output directory exists
    let output_dir = Path::new(OUTPUT_DIR);
    if !output_dir.exists() {
        // Create the directory and any necessary parent directories
        fs::create_dir_all(output_dir)?; 
    }

    // Create the file and write test transaction data in little-endian format
    let file_path = output_dir.join(FILE_NAME);
    let mut file = File::create(&file_path)?;
    
    // Test transaction data (all values set to 20 for testing)
    let transfer_id: u128 = 20;
    let block_number: u64 = 20;
    let tx_index: u64 = 0;
    let from_account: u128 = 20;
    let to_account: u128 = 20;
    let amount: u128 = 20;
    
    // Write all values in little-endian format
    file.write_all(&transfer_id.to_le_bytes())?;
    file.write_all(&block_number.to_le_bytes())?;
    file.write_all(&tx_index.to_le_bytes())?;
    file.write_all(&from_account.to_le_bytes())?;
    file.write_all(&to_account.to_le_bytes())?;
    file.write_all(&amount.to_le_bytes())?;

    Ok(())
}
