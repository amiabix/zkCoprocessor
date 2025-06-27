# zkCoprocessor

A Rust application that syncs Ethereum blockchain data to TigerBeetle and generates zero-knowledge proofs for transaction verification.

## 🎯 What This Project Does

- **Syncs Ethereum blocks** to TigerBeetle database for fast lookups
- **Generates real ZK proofs** using ZisK for transaction data integrity
- **Provides CLI tools** for testing, debugging, and proof generation
- **Supports both real and simulated proof modes** based on platform

## 📋 Prerequisites Check

Before starting, ensure you have these installed:

### Required (All Platforms)
- **Rust** (latest stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **TigerBeetle**: Download from [TigerBeetle releases](https://github.com/tigerbeetle/tigerbeetle/releases)
- **Ethereum RPC access**: Free endpoints available (see configuration)

### Optional (Linux Only)
- **ZisK**: For real cryptographic proofs (macOS uses simulation mode)

## 🚀 Quick Start Guide

### Step 1: Build the Project
```bash
# Clone the repository
git clone https://github.com/amiabix/zkCoprocessor.git
cd zkCoprocessor

# Build the project
cargo build --release
```

### Step 2: Start TigerBeetle Database
```bash
# Create and format the database
tigerbeetle format --cluster=0 --replica=0 --replica-count=1 0_0.tigerbeetle

# Start TigerBeetle server (keep this running in a separate terminal)
tigerbeetle start --addresses=3000 0_0.tigerbeetle
```

### Step 3: Test Your Setup
```bash
# Test TigerBeetle connection
cargo run -- test-tiger

# Test Ethereum RPC connection
cargo run -- test-eth

# If both tests pass, you're ready to proceed!
```

### Step 4: Sync Ethereum Data
```bash
# Sync a single block (recommended for testing)
cargo run -- sync-blocks --from 19000000 --to 19000000

# Sync multiple blocks (be patient, this takes time)
cargo run -- sync-blocks --from 19000000 --to 19000001
```

### Step 5: Verify Data and Generate Proofs
```bash
# Check what data was synced
cargo run -- debug --limit 10

# Generate a ZK proof for a transaction
cargo run -- prove-transaction --transfer-id 19000000000000

# Generate multiple proofs in batch
cargo run -- prove-batch --count 3
```

## 🔧 Detailed Installation Guide

### Installing TigerBeetle

**macOS:**
```bash
# Download and install TigerBeetle
curl -L https://github.com/tigerbeetle/tigerbeetle/releases/latest/download/tigerbeetle_$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/x86_64/amd64/')_0.16.45.tar.gz | tar -xz
sudo mv tigerbeetle /usr/local/bin/
```

**Linux:**
```bash
# Download and install TigerBeetle
curl -L https://github.com/tigerbeetle/tigerbeetle/releases/latest/download/tigerbeetle_$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/x86_64/amd64/')_0.16.45.tar.gz | tar -xz
sudo mv tigerbeetle /usr/local/bin/
```

### Installing ZisK (Linux Only)

**For real ZK proofs on Linux:**
```bash
# Run the setup script
chmod +x scripts/setup-zisk.sh
./scripts/setup-zisk.sh

# Or install manually
git clone https://github.com/0xPolygonHermez/zisk.git
cd zisk/cli
cargo build --release
cargo install --path .
```

**Note:** macOS users will automatically use simulation mode - no ZisK installation needed.

## 📖 Available Commands

### Connection Testing
```bash
# Test TigerBeetle connection
cargo run -- test-tiger

# Test Ethereum RPC connection
cargo run -- test-eth [--rpc-url <URL>]
```

### Data Synchronization
```bash
# Sync Ethereum blocks to TigerBeetle
cargo run -- sync-blocks --from <BLOCK> --to <BLOCK> [--rpc-url <URL>]

# Example: Sync block 19000000
cargo run -- sync-blocks --from 19000000 --to 19000000
```

### Data Inspection
```bash
# View stored data in TigerBeetle
cargo run -- debug [--limit <N>]

# Example: Show first 5 items
cargo run -- debug --limit 5
```

### ZK Proof Generation
```bash
# Generate ZK proof for a single transaction
cargo run -- prove-transaction --transfer-id <ID>

# Example: Generate proof for transfer 19000000000000
cargo run -- prove-transaction --transfer-id 19000000000000

# Generate multiple proofs in batch
cargo run -- prove-batch --count <N>

# Example: Generate 3 proofs
cargo run -- prove-batch --count 3
```

### Setup and Maintenance
```bash
# Setup ZisK project structure (Linux only)
cargo run -- setup-zisk

# Run performance benchmarks
cargo run -- benchmark [--num-transactions <N>]
```

## 🔍 Understanding the Output

### Sync Output
When you run `sync-blocks`, you'll see:
```
🔄 Syncing Ethereum blocks 19000000 to 19000000
📦 Fetching block 19000000
📦 Found 127 transactions in block 19000000
✅ Block 19000000: 127 transactions processed
🎉 Sync complete! 45 value transfers stored in TigerBeetle
```

### Debug Output
When you run `debug`, you'll see:
```
📊 TigerBeetle Contents:
========================
💸 Transfer 19000000000000: 1001 -> 1002 (1000000000000000000 wei, block 19000000)
💸 Transfer 19000000000002: 1003 -> 1004 (500000000000000000 wei, block 19000000)
...
```

### ZK Proof Output
When you run `prove-transaction`, you'll see:
```
🎯 Generating ZK proof for transfer_id: 19000000000000
🚀 Generating real ZisK proof using cargo-zisk...
✅ ZisK program built successfully
✅ Cryptographic proof generated successfully
✅ Proof verification successful

╭─────────────────────────────────────────────────────────────╮
│                🔍 ZK PROOF ANALYSIS                         │
╰─────────────────────────────────────────────────────────────╯

📋 PROOF DETAILS:
🎯 Transfer ID: 19000000000000
📦 Block Number: 19000000
🔐 Proof Type: zisk
✅ Valid: YES
⏱️  Generation Time: 2341ms
💾 Proof Size: 2048 bytes
📁 Proof File: /path/to/proof/vadcop_final_proof.json
```

## ⚙️ Configuration

### Default Settings
- **TigerBeetle**: `127.0.0.1:3000`
- **Ethereum RPC**: `https://eth.llamarpc.com`
- **Database file**: `0_0.tigerbeetle`

### Custom RPC Endpoints
You can use different Ethereum RPC endpoints:
```bash
# Use Infura
cargo run -- test-eth --rpc-url https://mainnet.infura.io/v3/YOUR_PROJECT_ID

# Use Alchemy
cargo run -- test-eth --rpc-url https://eth-mainnet.alchemyapi.io/v2/YOUR_API_KEY

# Use your own node
cargo run -- test-eth --rpc-url http://localhost:8545
```

## 🐛 Troubleshooting

### Common Issues

**"TigerBeetle connection failed"**
```bash
# Check if TigerBeetle is running
ps aux | grep tigerbeetle

# Restart TigerBeetle
tigerbeetle start --addresses=3000 0_0.tigerbeetle
```

**"No transfers found in debug"**
```bash
# You need to sync data first
cargo run -- sync-blocks --from 19000000 --to 19000000
cargo run -- debug --limit 5
```

**"Ethereum RPC connection failed"**
```bash
# Try a different RPC endpoint
cargo run -- test-eth --rpc-url https://eth.llamarpc.com
```

**"ZisK not found" (Linux)**
```bash
# Install ZisK
./scripts/setup-zisk.sh

# Or the system will automatically use simulation mode
```

### Platform-Specific Issues

**macOS Users:**
- ZisK doesn't support macOS yet
- The system automatically uses simulation mode
- All proof generation will work, but with simulated cryptographic guarantees
- This is perfect for development and testing

**Linux Users:**
- Full ZisK support available
- Install ZisK for real cryptographic proofs
- If ZisK fails, the system falls back to simulation mode

## 📊 Performance Tips

### For Large Datasets
```bash
# Use release builds for better performance
cargo build --release

# Sync blocks in smaller batches
cargo run -- sync-blocks --from 19000000 --to 19000005

# Monitor TigerBeetle memory usage
# TigerBeetle uses memory-mapped files, so ensure sufficient RAM
```

### For Proof Generation
```bash
# Generate proofs in batches for efficiency
cargo run -- prove-batch --count 10

# Use specific transfer IDs for targeted proofs
cargo run -- prove-transaction --transfer-id 19000000000000
```

## 🔮 What's Next?

### Current Capabilities
- ✅ Real Ethereum block synchronization
- ✅ TigerBeetle data storage and querying
- ✅ ZK proof generation (real on Linux, simulated on macOS)
- ✅ Batch proof processing
- ✅ Performance benchmarking

### Future Enhancements
- 🔄 Real transaction inclusion proofs (Merkle tree verification)
- 🔄 Parallel proof generation
- 🔄 Proof compression and optimization
- 🔄 Web interface for proof verification
- 🔄 Integration with other blockchains

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

This project is open source. See the LICENSE file for details.

---

**Ready to get started?** Follow the Quick Start Guide above! 🚀 