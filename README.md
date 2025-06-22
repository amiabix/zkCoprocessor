# zkCoprocessor

A Rust prototype that demonstrates synchronizing Ethereum blockchain data to TigerBeetle for high-performance transaction lookups. This project showcases the integration between Ethereum and TigerBeetle for potential zero-knowledge applications.

## 🚀 What It Does

- **Ethereum Integration**: Fetches real Ethereum blocks and transactions via RPC
- **TigerBeetle Sync**: Stores Ethereum transactions as TigerBeetle transfers for fast lookups
- **Performance Comparison**: Benchmarks TigerBeetle vs Ethereum RPC performance
- **CLI Interface**: Command-line tools for testing and debugging
- **Real Data Processing**: Handles actual Ethereum blockchain data

## 🏗️ Architecture

```
Ethereum Blockchain → zkCoprocessor → TigerBeetle Database
     (RPC)              (Rust)           (Fast Lookups)
```

- **Input**: Ethereum blocks via JSON-RPC
- **Processing**: Filters transactions with value > 0, transforms to TigerBeetle format
- **Output**: TigerBeetle transfers and accounts for high-performance queries

## 📋 Prerequisites

- **Rust** (latest stable version)
- **TigerBeetle** server running locally
- **Ethereum RPC endpoint** (public or private)

## 🛠️ Quick Start

1. **Clone and build**:
   ```bash
   git clone https://github.com/amiabix/zkCoprocessor.git
   cd zkCoprocessor
   cargo build --release
   ```

2. **Set up TigerBeetle**:
   ```bash
   # Format TigerBeetle database
   tigerbeetle format --cluster=0 --replica=0 --replica-count=1 0_0.tigerbeetle
   
   # Start TigerBeetle server
   tigerbeetle start --addresses=3000 0_0.tigerbeetle
   ```

3. **Test the setup**:
   ```bash
   # Test connections
   cargo run -- test-tiger
   cargo run -- test-eth
   
   # Sync some blocks
   cargo run -- sync-blocks --from 19000000 --to 19000000
   
   # Check what was stored
   cargo run -- debug --limit 5
   ```

## 🎯 Available Commands

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
```

### Performance Testing
```bash
# Run benchmarks
cargo run -- benchmark [--num-transactions <N>] [--include-ethereum]
```

### Debug & Inspection
```bash
# View stored data
cargo run -- debug [--limit <N>]

# Query specific items
cargo run -- query --account-id <ID>
cargo run -- query --transfer-id <ID>
```

## 📊 Performance Results

Example benchmark output:
```
🏁 zkCoprocessor Comprehensive Performance Benchmark
===================================================
Measuring: Transaction lookup performance
Testing: 50 transactions

📊 TigerBeetle Performance:
============================
  TigerBeetle Individual Lookups: 20 ops in 295ms (avg: 14.75ms/op, 67.6 ops/sec)
  TigerBeetle Batch Lookup: 1 ops in 15ms (avg: 15.00ms/op, 64.9 ops/sec)

💡 Why TigerBeetle Outperforms Ethereum RPC:
==============================================
✅ Local database vs network requests
✅ Optimized indexing for financial operations
✅ Batch operations support
✅ Memory-mapped files and caching
✅ Purpose-built for transaction processing
✅ No network latency or rate limiting
```

## 🏗️ Project Structure

```
zkcoprocessor/
├── Cargo.toml              # Project configuration and dependencies
├── .gitignore              # Git ignore rules
├── README.md               # This file
└── src/
    ├── main.rs             # Main application logic and CLI
    └── benchmark.rs        # Performance benchmarking module
```

### Key Components

#### `src/main.rs`
- CLI interface using `clap`
- TigerBeetle client integration
- Ethereum RPC client integration
- Block synchronization logic
- Debug and query functionality

#### `src/benchmark.rs`
- Performance benchmarking framework
- Individual vs batch operation testing
- Real transfer ID discovery
- Comparison between TigerBeetle and Ethereum RPC

## 🔧 Configuration

### Default Settings
- **TigerBeetle**: `127.0.0.1:3000`
- **Ethereum RPC**: `https://eth.llamarpc.com`

### Environment Variables
- `TIGERBEETLE_ADDRESS`: TigerBeetle server address
- `ETH_RPC_URL`: Ethereum RPC endpoint

## 🚨 Troubleshooting

### Common Issues

1. **TigerBeetle Connection Failed**
   - Ensure TigerBeetle server is running: `tigerbeetle start --addresses=3000 0_0.tigerbeetle`
   - Check if port 3000 is available

2. **Ethereum RPC Connection Failed**
   - Verify RPC endpoint is accessible
   - Try a different RPC provider
   - Check network connectivity

3. **No Transfers Found in Debug**
   - Run sync operation first: `cargo run -- sync-blocks --from 19000000 --to 19000000`
   - Check sync logs for errors

4. **Large File Size Issues**
   - TigerBeetle database files are excluded from git (see `.gitignore`)
   - Database files are created locally when running the application

### Debug Commands

```bash
# Check what's stored in TigerBeetle
cargo run -- debug --limit 20

# Test with a small block range
cargo run -- sync-blocks --from 19000000 --to 19000000

# Run benchmarks with fewer transactions
cargo run -- benchmark --num-transactions 10
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and test thoroughly
4. Commit your changes: `git commit -m 'Add feature'`
5. Push to the branch: `git push origin feature-name`
6. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [TigerBeetle](https://github.com/tigerbeetle/tigerbeetle) - High-performance financial database
- [Ethers.rs](https://github.com/gakonst/ethers-rs) - Ethereum library for Rust
- [Clap](https://github.com/clap-rs/clap) - Command line argument parsing

---

**zkCoprocessor** - A prototype bridging Ethereum and TigerBeetle 🚀 