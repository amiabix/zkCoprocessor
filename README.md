# zkCoprocessor

A simple Rust demo that syncs Ethereum blockchain data to TigerBeetle. This project shows how to fetch Ethereum blocks via RPC and store transactions as TigerBeetle transfers for fast lookups.

## What It Does

- Fetches Ethereum blocks and transactions via JSON-RPC
- Stores transactions with value > 0 as TigerBeetle transfers
- Provides basic CLI commands for testing and debugging
- Includes simple performance benchmarks
- **Generates real zero-knowledge proofs using ZisK**

## Prerequisites

- Rust (latest stable)
- TigerBeetle server
- Ethereum RPC endpoint
- **ZisK (for real ZK proofs on Linux)**

## Quick Start

1. **Build the project**:
   ```bash
   cargo build --release
   ```

2. **Set up TigerBeetle**:
   ```bash
   # Format database
   tigerbeetle format --cluster=0 --replica=0 --replica-count=1 0_0.tigerbeetle
   
   # Start server
   tigerbeetle start --addresses=3000 0_0.tigerbeetle
   ```

3. **Test it**:
   ```bash
   # Test connections
   cargo run -- test-tiger
   cargo run -- test-eth
   
   # Sync a block
   cargo run -- sync-blocks --from 19000000 --to 19000000
   
   # Check what was stored
   cargo run -- debug --limit 5
   
   # Generate ZK proofs
   cargo run -- prove-transaction --transfer-id 19000000000000
   cargo run -- prove-batch --count 3
   ```

## AWS Deployment Guide

### 1. Launch EC2 Instance
```bash
# Recommended: Ubuntu 22.04 LTS
# Instance type: t3.medium or larger
# Storage: 20GB+ for TigerBeetle database
```

### 2. Connect and Update System
```bash
ssh -i your-key.pem ubuntu@your-instance-ip
sudo apt update && sudo apt upgrade -y
```

### 3. Install Dependencies
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install build tools
sudo apt install -y build-essential pkg-config libssl-dev

# Install TigerBeetle
curl -L https://github.com/tigerbeetle/tigerbeetle/releases/latest/download/tigerbeetle_$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/x86_64/amd64/')_0.16.45.tar.gz | tar -xz
sudo mv tigerbeetle /usr/local/bin/

# Install ZisK (for real ZK proofs)
curl -L https://github.com/delendum-xyz/zisk/releases/latest/download/zisk-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m | sed 's/x86_64/x64/') -o zisk
chmod +x zisk
sudo mv zisk /usr/local/bin/
```

### 4. Clone and Build Project
```bash
git clone https://github.com/amiabix/zkCoprocessor.git
cd zkCoprocessor
cargo build --release
```

### 5. Set Up TigerBeetle Database
```bash
# Format database
tigerbeetle format --cluster=0 --replica=0 --replica-count=1 0_0.tigerbeetle

# Start TigerBeetle server (in background)
nohup tigerbeetle start --addresses=3000 0_0.tigerbeetle > tigerbeetle.log 2>&1 &
```

### 6. Test the Setup
```bash
# Test TigerBeetle connection
./target/release/zkcoprocessor test-tiger

# Test Ethereum RPC
./target/release/zkcoprocessor test-eth

# Sync some blocks
./target/release/zkcoprocessor sync-blocks --from 19000000 --to 19000000

# Generate real ZK proof
./target/release/zkcoprocessor prove-transaction --transfer-id 19000000000000
```

### 7. Production Setup (Optional)
```bash
# Create systemd service for TigerBeetle
sudo tee /etc/systemd/system/tigerbeetle.service > /dev/null <<EOF
[Unit]
Description=TigerBeetle Database Server
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu
ExecStart=/usr/local/bin/tigerbeetle start --addresses=3000 0_0.tigerbeetle
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl enable tigerbeetle
sudo systemctl start tigerbeetle
```

## Commands

### Connection Tests
```bash
# Test TigerBeetle connection
cargo run -- test-tiger

# Test Ethereum RPC connection
cargo run -- test-eth [--rpc-url <URL>]
```

### Data Sync
```bash
# Sync Ethereum blocks to TigerBeetle
cargo run -- sync-blocks --from <BLOCK> --to <BLOCK> [--rpc-url <URL>]
```

### Debug & Query
```bash
# View stored data
cargo run -- debug [--limit <N>]

# Query specific items
cargo run -- query --account-id <ID>
cargo run -- query --transfer-id <ID>
```

### ZK Proof Generation
```bash
# Generate ZK proof for a single transaction
cargo run -- prove-transaction --transfer-id <ID>

# Generate ZK proofs for multiple transactions
cargo run -- prove-batch --count <N>
```

### Benchmarks
```bash
# Run performance tests
cargo run -- benchmark [--num-transactions <N>] [--include-ethereum]
```

## ZK Proof System

### Current Status
- **Real ZK Mode**: Now generates real zero-knowledge proofs using ZisK on Linux
- **Platform Support**: Full ZisK support on Linux, simulation mode on macOS
- **Proof Types**: Real cryptographic proofs with actual transaction data

### Proof Features
- **Real Transaction Data**: Uses actual TigerBeetle transaction data
- **Cryptographic Verification**: Generates verifiable zero-knowledge proofs
- **Detailed Breakdown**: Shows exactly what each public output represents
- **Batch Processing**: Generate multiple proofs efficiently

### Proof Output Format
The system generates 9 public outputs that prove transaction inclusion:
- `public 0-3`: Parts of the real transfer_id (e.g., `19000000000000`)
- `public 4-5`: Parts of the real block_number (e.g., `19000000`)
- `public 6-7`: Real cryptographic proof hash
- `public 8`: Validity flag (1=valid, 0=invalid)

### Example Proof Output
```
🔍 ZK Proof Breakdown:
=======================
public 0: 0xc8403000  ← transfer_id part 1 (bytes 0-3)
public 1: 0x00001147  ← transfer_id part 2 (bytes 4-7)
public 2: 0x00000000  ← transfer_id part 3 (bytes 8-11)
public 3: 0x00000000  ← transfer_id part 4 (bytes 12-15)
public 4: 0x0121eac0  ← block_number part 1 (bytes 0-3)
public 5: 0x00000000  ← block_number part 2 (bytes 4-7)
public 6: 0x82edac32  ← proof_hash part 1 (bytes 0-3)
public 7: 0x915bde57  ← proof_hash part 2 (bytes 4-7)
public 8: 0x00000001  ← validity flag (1=valid, 0=invalid)
```

## How It Works

1. **Ethereum RPC**: Fetches blocks and transactions from Ethereum
2. **Filtering**: Only processes transactions with value > 0
3. **TigerBeetle Storage**: Creates accounts and transfers in TigerBeetle
4. **ID Mapping**: Converts Ethereum addresses to u128 IDs for TigerBeetle
5. **ZK Proofs**: Generates real cryptographic proofs using ZisK

## Project Structure

```
src/
├── main.rs          # Main CLI and sync logic
├── benchmark.rs     # Performance benchmarking
└── zk/             # ZK proof system
    ├── mod.rs      # Module exports
    └── prover.rs   # ZK proof logic
zisk-tx-proof/      # ZisK program for transaction proofs
    ├── Cargo.toml
    └── src/main.rs
```

## Configuration

- **TigerBeetle**: `127.0.0.1:3000` (default)
- **Ethereum RPC**: `https://eth.llamarpc.com` (default)
- **ZisK**: Automatically detected and used on supported platforms

## Troubleshooting

### Common Issues
- **No transfers found**: Run sync first, then debug
- **Connection errors**: Check if TigerBeetle is running
- **RPC errors**: Try a different Ethereum RPC endpoint
- **ZK proof errors**: Ensure ZisK is installed on Linux

### AWS-Specific Issues
- **Permission denied**: Check security groups allow port 3000
- **Out of memory**: Use larger instance type for large datasets
- **Disk space**: Monitor TigerBeetle database size
- **Network timeouts**: Use closer RPC endpoints

### ZK Proof Issues
- **Dummy data**: Run from main directory, not zisk-tx-proof subdirectory
- **ZisK not found**: Install ZisK binary in PATH
- **Build errors**: Ensure all dependencies are installed

## Development Notes

### ZisK Integration
- **Linux**: Full real ZK proof support
- **macOS**: Simulation mode (ZisK doesn't support macOS yet)
- **Automatic Detection**: System detects platform and uses appropriate mode
- **Real Data**: Always uses actual TigerBeetle transaction data

### Performance Tips
- Use `--release` builds for production
- Monitor TigerBeetle memory usage
- Use efficient RPC endpoints
- Consider batch operations for large datasets

---

A production-ready demo of Ethereum → TigerBeetle integration with real zero-knowledge proofs 🚀 