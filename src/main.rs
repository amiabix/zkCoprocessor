use anyhow::Result;
use clap::{Parser, Subcommand};
use tigerbeetle_unofficial::{Account, Transfer};
use tracing::{info, warn};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use ethers::{
    providers::{Http, Provider, Middleware},
    types::BlockNumber,
};

mod zk;
mod benchmark;
use benchmark::TransactionLookupBenchmark;

#[derive(Parser)]
#[command(name = "zkcoprocessor")]
#[command(about = "Ethereum transaction verification with ZK proofs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Test TigerBeetle connection
    TestTiger,
    
    /// Test Ethereum connection  
    TestEth {
        #[arg(long, default_value = "https://eth.llamarpc.com")]
        rpc_url: String,
    },
    
    /// Sync Ethereum blocks to TigerBeetle
    SyncBlocks {
        #[arg(long, default_value = "https://eth.llamarpc.com")]
        rpc_url: String,
        #[arg(long)]
        from: u64,
        #[arg(long)]  
        to: u64,
    },
    
    /// Run performance benchmarks
    Benchmark {
        #[arg(long, default_value = "50")]
        num_transactions: usize,
        #[arg(long)]
        include_ethereum: bool,
        #[arg(long, default_value = "https://eth.llamarpc.com")]
        rpc_url: String,
    },
    
    /// Debug: Show stored transfers
    Debug {
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    
    /// Query data
    Query {
        #[arg(long)]
        account_id: Option<u128>,
        #[arg(long)]
        transfer_id: Option<u128>,
    },
    
    /// Generate ZK proof for transaction inclusion
    ProveTransaction {
        /// Transfer ID to prove
        #[arg(long)]
        transfer_id: u128,
    },
    
    /// Batch generate ZK proofs
    ProveBatch {
        /// Number of transactions to prove
        #[arg(long, default_value = "5")]
        count: usize,
    },
}

struct TigerBeetleClient {
    client: tigerbeetle_unofficial::Client,
}

impl TigerBeetleClient {
    fn new() -> Result<Self> {
        info!("🐅 Connecting to TigerBeetle at 127.0.0.1:3000");
        
        let client = tigerbeetle_unofficial::Client::new(
            0, // cluster_id
            "127.0.0.1:3000", // address
        )?;
        
        info!("✅ Connected to TigerBeetle!");
        Ok(Self { client })
    }
    
    async fn create_account(&mut self, id: u128, code: u16) -> Result<()> {
        let account = Account::new(id, 1, code);
        
        match self.client.create_accounts(vec![account]).await {
            Ok(_) => {
                info!("✅ Created account: {}", id);
                Ok(())
            },
            Err(e) => {
                // Account might already exist, that's OK
                warn!("Account {} may already exist: {}", id, e);
                Ok(())
            },
        }
    }
    
    async fn create_transfer(
        &mut self,
        id: u128,
        debit_account_id: u128,
        credit_account_id: u128,
        amount: u128,
        block_number: u128,
        gas_used: u64,
    ) -> Result<()> {
        let transfer = Transfer::new(id)
            .with_debit_account_id(debit_account_id)
            .with_credit_account_id(credit_account_id)
            .with_amount(amount)
            .with_user_data_128(block_number)
            .with_user_data_64(gas_used)
            .with_ledger(1)
            .with_code(1);
        
        match self.client.create_transfers(vec![transfer]).await {
            Ok(_) => {
                info!("✅ Created transfer: {} -> {} ({} wei)", 
                      debit_account_id, credit_account_id, amount);
                Ok(())
            },
            Err(e) => {
                warn!("Transfer {} may already exist: {}", id, e);
                Ok(())
            },
        }
    }
    
    async fn lookup_accounts(&mut self, ids: &[u128]) -> Result<Vec<Account>> {
        self.client.lookup_accounts(ids.to_vec())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to lookup accounts: {}", e))
    }
    
    async fn lookup_transfers(&mut self, ids: &[u128]) -> Result<Vec<Transfer>> {
        self.client.lookup_transfers(ids.to_vec())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to lookup transfers: {}", e))
    }
}

#[derive(Debug, Clone)]
struct EthTransaction {
    hash: String,
    block_number: u64,
    from: String,
    to: Option<String>,
    value_wei: String,
    gas_used: u64,
}

struct EthereumClient {
    provider: Provider<Http>,
}

impl EthereumClient {
    async fn new(rpc_url: &str) -> Result<Self> {
        info!("🔗 Connecting to Ethereum: {}", rpc_url);
        let provider = Provider::<Http>::try_from(rpc_url)?;
        
        let latest_block = provider.get_block_number().await?;
        info!("✅ Connected! Latest block: {}", latest_block);
        
        Ok(Self { provider })
    }
    
    async fn get_block_transactions(&self, block_number: u64) -> Result<Vec<EthTransaction>> {
        info!("📦 Fetching block {}", block_number);
        
        let block = self.provider
            .get_block_with_txs(BlockNumber::Number(block_number.into()))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Block {} not found", block_number))?;
            
        let transactions: Vec<EthTransaction> = block.transactions
            .into_iter()
            .map(|tx| EthTransaction {
                hash: format!("{:?}", tx.hash),
                block_number,
                from: format!("{:?}", tx.from),
                to: tx.to.map(|addr| format!("{:?}", addr)),
                value_wei: tx.value.to_string(),
                gas_used: tx.gas.as_u64(),
            })
            .collect();
            
        info!("📦 Found {} transactions in block {}", transactions.len(), block_number);
        Ok(transactions)
    }
}

impl EthTransaction {
    fn hash_to_u128(&self) -> u128 {
        let hash_str = self.hash.trim_start_matches("0x");
        let bytes = hex::decode(hash_str).unwrap_or_default();
        
        if bytes.len() >= 16 {
            let mut id_bytes = [0u8; 16];
            id_bytes.copy_from_slice(&bytes[bytes.len()-16..]);
            u128::from_be_bytes(id_bytes)
        } else {
            let mut hasher = DefaultHasher::new();
            self.hash.hash(&mut hasher);
            hasher.finish() as u128
        }
    }
    
    fn from_address_to_u128(&self) -> u128 {
        self.address_to_u128(&self.from)
    }
    
    fn to_address_to_u128(&self) -> u128 {
        self.to.as_ref()
            .map(|addr| self.address_to_u128(addr))
            .unwrap_or(0)
    }
    
    fn address_to_u128(&self, addr: &str) -> u128 {
        let addr_str = addr.trim_start_matches("0x");
        let bytes = hex::decode(addr_str).unwrap_or_default();
        
        if bytes.len() >= 16 {
            let mut addr_bytes = [0u8; 16];
            addr_bytes.copy_from_slice(&bytes[bytes.len()-16..]);
            u128::from_be_bytes(addr_bytes)
        } else {
            let mut hasher = DefaultHasher::new();
            addr.hash(&mut hasher);
            hasher.finish() as u128
        }
    }
    
    fn value_to_u128(&self) -> u128 {
        self.value_wei.parse::<u128>().unwrap_or(0)
    }
}

async fn test_ethereum(rpc_url: &str) -> Result<()> {
    info!("🧪 Testing Ethereum connection");
    
    let client = EthereumClient::new(rpc_url).await?;
    let latest_block = client.provider.get_block_number().await?.as_u64();
    let test_block = latest_block - 10;
    
    let transactions = client.get_block_transactions(test_block).await?;
    
    if !transactions.is_empty() {
        let tx = &transactions[0];
        println!("✅ Sample transaction from block {}:", test_block);
        println!("   Hash: {}", tx.hash);
        println!("   From: {}", tx.from);
        println!("   To: {:?}", tx.to);
        println!("   Value: {} wei ({} ETH)", tx.value_wei, 
                tx.value_to_u128() as f64 / 1e18);
    }
    
    info!("🎉 Ethereum test completed!");
    Ok(())
}

// Helper function to convert Ethereum address to account ID
fn address_to_account_id(address: &str) -> u128 {
    let addr_str = address.trim_start_matches("0x");
    let bytes = hex::decode(addr_str).unwrap_or_default();
    
    if bytes.len() >= 16 {
        let mut addr_bytes = [0u8; 16];
        addr_bytes.copy_from_slice(&bytes[bytes.len()-16..]);
        u128::from_be_bytes(addr_bytes)
    } else {
        let mut hasher = DefaultHasher::new();
        address.hash(&mut hasher);
        hasher.finish() as u128
    }
}

// Helper function to ensure account exists
async fn ensure_account_exists(tb_client: &mut TigerBeetleClient, account_id: u128) -> Result<()> {
    if account_id == 0 {
        return Ok(());
    }
    
    match tb_client.create_account(account_id, 1).await {
        Ok(_) => Ok(()),
        Err(_) => Ok(()), // Account might already exist
    }
}

async fn process_transaction(
    tb_client: &mut TigerBeetleClient,
    block_number: u64,
    tx_index: usize,
    eth_tx: &EthTransaction,
) -> Result<bool> {
    // Only process transactions with value > 0
    if eth_tx.value_to_u128() == 0 {
        return Ok(false);
    }

    // Generate consistent transfer ID: block_number * 1000000 + tx_index
    let transfer_id = (block_number as u128) * 1000000 + (tx_index as u128);
    
    info!("Processing tx {} in block {} -> transfer_id: {}", 
          tx_index, block_number, transfer_id);

    // Convert addresses to account IDs
    let from_account = eth_tx.from_address_to_u128();
    let to_account = eth_tx.to_address_to_u128();

    // Ensure accounts exist
    ensure_account_exists(tb_client, from_account).await?;
    if to_account != 0 {
        ensure_account_exists(tb_client, to_account).await?;
    }

    // Create transfer using the TigerBeetle client method
    match tb_client.create_transfer(
        transfer_id,
        from_account,
        to_account,
        eth_tx.value_to_u128(),
        block_number as u128,
        eth_tx.gas_used,
    ).await {
        Ok(_) => {
            info!("✅ Created transfer: {} (amount: {})", transfer_id, eth_tx.value_to_u128());
            Ok(true)
        },
        Err(e) => {
            warn!("Transfer {} may already exist: {}", transfer_id, e);
            Ok(false) // Don't fail the whole process for duplicates
        }
    }
}

async fn sync_ethereum_blocks(rpc_url: &str, from_block: u64, to_block: u64) -> Result<()> {
    info!("🔄 Syncing Ethereum blocks {} to {}", from_block, to_block);
    
    let eth_client = EthereumClient::new(rpc_url).await?;
    let mut tb_client = TigerBeetleClient::new()?;
    
    let mut total_transactions = 0;
    
    for block_num in from_block..=to_block {
        let eth_transactions = eth_client.get_block_transactions(block_num).await?;
        
        for (i, eth_tx) in eth_transactions.iter().enumerate() {
            if process_transaction(&mut tb_client, block_num, i, eth_tx).await? {
                total_transactions += 1;
            }
        }
        
        println!("✅ Block {}: {} transactions processed", 
                block_num, eth_transactions.len());
    }
    
    info!("🎉 Sync complete! {} value transfers stored in TigerBeetle", total_transactions);
    Ok(())
}

async fn test_tigerbeetle() -> Result<()> {
    info!("🧪 Testing TigerBeetle operations");
    
    let mut client = TigerBeetleClient::new()?;
    
    // Create test accounts
    info!("Creating test accounts...");
    client.create_account(1001, 1).await?; // Alice
    client.create_account(1002, 1).await?; // Bob
    client.create_account(1003, 1).await?; // Charlie
    
    // Create test transfers
    info!("Creating test transfers...");
    client.create_transfer(
        2001,
        1001, // Alice
        1002, // Bob
        1000000000000000000, // 1 ETH
        19000000, // block
        21000,    // gas
    ).await?;
    
    client.create_transfer(
        2002,
        1002, // Bob
        1003, // Charlie
        500000000000000000, // 0.5 ETH
        19000000,
        21000,
    ).await?;
    
    // Query accounts
    info!("Querying accounts...");
    let accounts = client.lookup_accounts(&[1001, 1002, 1003]).await?;
    println!("📊 Account Balances:");
    for account in &accounts {
        let balance = account.credits_posted().saturating_sub(account.debits_posted());
        println!("  Account {}: {} ETH", 
                account.id(), 
                balance as f64 / 1e18);
    }
    
    // Query transfers
    info!("Querying transfers...");
    let transfers = client.lookup_transfers(&[2001, 2002]).await?;
    println!("💸 Transfers:");
    for transfer in &transfers {
        println!("  Transfer {}: {} -> {} ({} ETH, block {})", 
                transfer.id(),
                transfer.debit_account_id(),
                transfer.credit_account_id(),
                transfer.amount() as f64 / 1e18,
                transfer.user_data_128());
    }
    
    info!("🎉 TigerBeetle test completed successfully!");
    Ok(())
}

async fn run_benchmarks(num_transactions: usize, include_ethereum: bool, rpc_url: &str) -> Result<()> {
    let mut benchmark = TransactionLookupBenchmark::new();
    
    // Initialize TigerBeetle
    benchmark.init_tigerbeetle().await?;
    
    // Initialize Ethereum if requested
    if include_ethereum {
        benchmark.init_ethereum(rpc_url).await?;
    }
    
    // Run comprehensive benchmark
    benchmark.run_comprehensive_benchmark(num_transactions).await?;
    
    Ok(())
}

async fn debug_tigerbeetle_contents(limit: usize) -> Result<()> {
    info!("🔍 Debugging TigerBeetle contents (showing up to {} items)", limit);
    
    let mut client = TigerBeetleClient::new()?;
    
    println!("\n📊 TigerBeetle Contents:");
    println!("========================");
    
    // Try to find transfers in the expected range
    let mut found_transfers = 0;
    let mut found_accounts = 0;
    
    // Search for transfers in the range we know from sync
    for block in 19000000u128..19000002u128 {
        for tx_index in 0u128..1000u128 {
            let transfer_id = block * 1000000 + tx_index;
            
            match client.lookup_transfers(&[transfer_id]).await {
                Ok(transfers) if !transfers.is_empty() => {
                    for transfer in transfers {
                        println!("💸 Transfer {}: {} -> {} ({} wei, block {})", 
                                transfer.id(),
                                transfer.debit_account_id(),
                                transfer.credit_account_id(),
                                transfer.amount(),
                                transfer.user_data_128());
                        found_transfers += 1;
                        
                        if found_transfers >= limit {
                            break;
                        }
                    }
                },
                _ => {}
            }
            
            if found_transfers >= limit {
                break;
            }
        }
        
        if found_transfers >= limit {
            break;
        }
    }
    
    // Also try to find some accounts
    println!("\n👤 Sample Accounts:");
    println!("===================");
    
    // Search for accounts in a reasonable range
    for account_id in 1u128..1000u128 {
        match client.lookup_accounts(&[account_id]).await {
            Ok(accounts) if !accounts.is_empty() => {
                for account in accounts {
                    let balance = account.credits_posted().saturating_sub(account.debits_posted());
                    println!("👤 Account {}: {} credits, {} debits (balance: {} wei)", 
                            account.id(),
                            account.credits_posted(),
                            account.debits_posted(),
                            balance);
                    found_accounts += 1;
                    
                    if found_accounts >= limit {
                        break;
                    }
                }
            },
            _ => {}
        }
        
        if found_accounts >= limit {
            break;
        }
    }
    
    if found_transfers == 0 && found_accounts == 0 {
        println!("❌ No data found in TigerBeetle!");
        println!("💡 Try running 'sync-blocks' first to populate the database.");
    } else {
        println!("\n✅ Found {} transfers and {} accounts", found_transfers, found_accounts);
    }
    
    Ok(())
}

async fn debug_transfers(limit: usize) -> Result<()> {
    info!("🔍 Debugging stored transfers and accounts (limit: {})", limit);
    
    let mut tb_client = TigerBeetleClient::new()?;
    
    println!("🔍 Checking what's actually stored in TigerBeetle...");
    
    // Use the ACTUAL transfer IDs from your sync logs
    println!("\n📋 Checking transfers with ACTUAL IDs from sync:");
    
    let actual_transfer_ids_block_19000000 = vec![
        19000000000000u128, 19000000000002u128, 19000000000003u128, 19000000000005u128,
        19000000000006u128, 19000000000007u128, 19000000000008u128, 19000000000009u128,
        19000000000012u128, 19000000000016u128, 19000000000022u128, 19000000000023u128,
        19000000000025u128, 19000000000027u128, 19000000000028u128, 19000000000045u128,
        19000000000046u128, 19000000000048u128, 19000000000051u128, 19000000000052u128,
        19000000000055u128, 19000000000056u128, 19000000000058u128, 19000000000059u128,
        19000000000061u128, 19000000000064u128, 19000000000065u128, 19000000000066u128,
        19000000000068u128, 19000000000073u128, 19000000000074u128, 19000000000076u128,
        19000000000077u128, 19000000000082u128, 19000000000087u128, 19000000000089u128,
        19000000000090u128, 19000000000091u128, 19000000000094u128, 19000000000095u128,
        19000000000096u128, 19000000000097u128, 19000000000101u128, 19000000000103u128,
        19000000000104u128, 19000000000105u128, 19000000000106u128, 19000000000107u128,
        19000000000108u128, 19000000000109u128, 19000000000112u128, 19000000000113u128,
        19000000000114u128, 19000000000115u128, 19000000000116u128, 19000000000119u128,
        19000000000122u128, 19000000000123u128, 19000000000124u128, 19000000000125u128,
        19000000000126u128, 19000000000129u128, 19000000000130u128, 19000000000131u128,
        19000000000132u128,
    ];

    let actual_transfer_ids_block_19000001 = vec![
        19000001000004u128, 19000001000006u128, 19000001000008u128, 19000001000011u128,
        19000001000012u128, 19000001000015u128, 19000001000016u128, 19000001000027u128,
        19000001000028u128, 19000001000033u128, 19000001000034u128, 19000001000035u128,
        19000001000037u128, 19000001000040u128, 19000001000042u128, 19000001000043u128,
        19000001000045u128, 19000001000046u128, 19000001000047u128, 19000001000048u128,
        19000001000049u128, 19000001000050u128, 19000001000055u128, 19000001000056u128,
        19000001000058u128, 19000001000062u128, 19000001000064u128, 19000001000068u128,
        19000001000069u128, 19000001000070u128, 19000001000071u128, 19000001000076u128,
        19000001000077u128, 19000001000078u128, 19000001000079u128, 19000001000080u128,
        19000001000081u128, 19000001000089u128, 19000001000092u128, 19000001000094u128,
        19000001000102u128, 19000001000103u128, 19000001000106u128, 19000001000107u128,
        19000001000109u128, 19000001000111u128, 19000001000114u128, 19000001000115u128,
        19000001000119u128, 19000001000120u128, 19000001000121u128, 19000001000123u128,
        19000001000124u128, 19000001000126u128, 19000001000128u128, 19000001000133u128,
        19000001000135u128, 19000001000136u128, 19000001000137u128, 19000001000143u128,
        19000001000148u128, 19000001000149u128, 19000001000151u128, 19000001000152u128,
        19000001000154u128, 19000001000155u128, 19000001000157u128, 19000001000158u128,
        19000001000160u128, 19000001000163u128,
    ];

    // Combine and test first 'limit' transfers
    let mut all_transfer_ids = actual_transfer_ids_block_19000000;
    all_transfer_ids.extend(actual_transfer_ids_block_19000001);
    
    let test_ids = &all_transfer_ids[0..limit.min(all_transfer_ids.len())];
    
    println!("Testing {} actual transfer IDs from sync logs:", test_ids.len());
    
    let mut found_count = 0;
    let mut accounts_found = std::collections::HashSet::new();
    
    for &transfer_id in test_ids {
        match tb_client.lookup_transfers(&[transfer_id]).await {
            Ok(transfers) if !transfers.is_empty() => {
                let transfer = &transfers[0];
                println!("✅ Found transfer: {}", transfer_id);
                println!("   Amount: {}", transfer.amount());
                println!("   From Account: {}", transfer.debit_account_id());
                println!("   To Account: {}", transfer.credit_account_id());
                println!("   Block: {}", transfer.user_data_128());
                println!();
                found_count += 1;
                accounts_found.insert(transfer.debit_account_id());
                accounts_found.insert(transfer.credit_account_id());
            },
            Ok(_) => {
                println!("❌ Transfer {} not found", transfer_id);
            },
            Err(e) => {
                println!("🚨 Error looking up {}: {}", transfer_id, e);
            }
        }
    }
    
    println!("📊 Summary:");
    println!("   Found {}/{} transfers", found_count, test_ids.len());
    println!("   {} unique accounts involved", accounts_found.len());
    
    if found_count == 0 {
        println!("🚨 No transfers found! The sync may have failed silently.");
        println!("   Try running: cargo run -- sync-blocks --from 19000000 --to 19000000");
    } else {
        println!("🎉 Data found! zkCoprocessor is working correctly!");
    }
    
    Ok(())
}

/// Generate ZK proof for a specific transaction
async fn cmd_prove_transaction(transfer_id: u128) -> Result<()> {
    info!("🎯 Generating ZK proof for transfer_id: {}", transfer_id);
    
    let mut tb_client = TigerBeetleClient::new()?;
    let proof = zk::generate_zk_proof(&mut tb_client.client, transfer_id).await?;
    
    // Display results
    println!("\n🔐 ZK Proof Results:");
    println!("=====================================");
    println!("Transfer ID: {}", proof.transfer_id);
    println!("Block Number: {}", proof.block_number);
    println!("Proof Hash: {}", hex::encode(proof.inclusion_proof_hash));
    println!("Proof Type: {}", proof.proof_type);
    println!("Valid: {}", if proof.is_valid { "✅ YES" } else { "❌ NO" });
    
    if let Some(proof_path) = &proof.proof_path {
        println!("Proof Path: {}", proof_path);
    }
    
    if proof.is_valid {
        println!("\n🎉 Transaction inclusion successfully proven!");
    }
    
    Ok(())
}

/// Generate proofs for multiple transactions
async fn cmd_prove_batch(count: usize) -> Result<()> {
    info!("🎯 Generating {} ZK proofs in batch", count);
    
    let mut tb_client = TigerBeetleClient::new()?;
    
    // Use the actual transfer IDs from sync logs
    let known_transfer_ids = vec![
        // Block 19000000
        19000000000000u128, 19000000000002u128, 19000000000003u128, 19000000000005u128,
        19000000000006u128, 19000000000007u128, 19000000000008u128, 19000000000009u128,
        19000000000012u128, 19000000000016u128, 19000000000022u128, 19000000000023u128,
        19000000000025u128, 19000000000027u128, 19000000000028u128, 19000000000045u128,
        19000000000046u128, 19000000000048u128, 19000000000051u128, 19000000000052u128,
        19000000000055u128, 19000000000056u128, 19000000000058u128, 19000000000059u128,
        19000000000061u128, 19000000000064u128, 19000000000065u128, 19000000000066u128,
        19000000000068u128, 19000000000073u128, 19000000000074u128, 19000000000076u128,
        19000000000077u128, 19000000000082u128, 19000000000087u128, 19000000000089u128,
        19000000000090u128, 19000000000091u128, 19000000000094u128, 19000000000095u128,
        19000000000096u128, 19000000000097u128, 19000000000101u128, 19000000000103u128,
        19000000000104u128, 19000000000105u128, 19000000000106u128, 19000000000107u128,
        19000000000108u128, 19000000000109u128, 19000000000112u128, 19000000000113u128,
        19000000000114u128, 19000000000115u128, 19000000000116u128, 19000000000119u128,
        19000000000122u128, 19000000000123u128, 19000000000124u128, 19000000000125u128,
        19000000000126u128, 19000000000129u128, 19000000000130u128, 19000000000131u128,
        19000000000132u128,
    ];
    
    let transfer_ids = &known_transfer_ids[0..count.min(known_transfer_ids.len())];
    
    println!("\n🔐 Batch ZK Proof Generation:");
    println!("================================");
    
    let mut successful_proofs = 0;
    
    for (i, &transfer_id) in transfer_ids.iter().enumerate() {
        println!("\n📝 Proof {}/{}: Transfer ID {}", i + 1, transfer_ids.len(), transfer_id);
        
        match zk::generate_zk_proof(&mut tb_client.client, transfer_id).await {
            Ok(proof) if proof.is_valid => {
                successful_proofs += 1;
                println!("  ✅ Generated valid proof");
            }
            Ok(_) => {
                println!("  ❌ Invalid proof generated");
            }
            Err(e) => {
                println!("  🚨 Failed: {}", e);
            }
        }
    }
    
    println!("\n📊 Results: {}/{} successful proofs", successful_proofs, transfer_ids.len());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();
    
    info!("🚀 zkCoprocessor v0.1.0 (Real TigerBeetle)");
    
    match cli.command {
        Commands::TestTiger => {
            test_tigerbeetle().await?;
        },
        Commands::TestEth { rpc_url } => {
            test_ethereum(&rpc_url).await?;
        },
        Commands::SyncBlocks { rpc_url, from, to } => {
            sync_ethereum_blocks(&rpc_url, from, to).await?;
        },
        Commands::Benchmark { num_transactions, include_ethereum, rpc_url } => {
            run_benchmarks(num_transactions, include_ethereum, &rpc_url).await?;
        },
        Commands::Debug { limit } => {
            debug_transfers(limit).await?;
        },
        Commands::Query { account_id: _, transfer_id: _ } => {
            info!("Query feature coming in Step 3!");
        },
        Commands::ProveTransaction { transfer_id } => {
            cmd_prove_transaction(transfer_id).await?;
        },
        Commands::ProveBatch { count } => {
            cmd_prove_batch(count).await?;
        },
    }
    
    Ok(())
}