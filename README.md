# zkCoprocessor

A high-performance Rust application that synchronizes Ethereum blockchain data to TigerBeetle, enabling fast transaction lookups and financial data processing for zero-knowledge applications.

## 🚀 Features

- **Ethereum Integration**: Fetches real Ethereum blocks and transactions via RPC
- **TigerBeetle Sync**: Stores Ethereum transactions as TigerBeetle transfers for fast lookups
- **Performance Benchmarking**: Compare TigerBeetle vs Ethereum RPC performance
- **CLI Interface**: Easy-to-use command-line interface for all operations
- **Debug Tools**: Inspect stored data and troubleshoot sync issues
- **Real-time Processing**: Handles live Ethereum data with proper error handling

## 🏗️ Architecture

```
Ethereum Blockchain → zkCoprocessor → TigerBeetle Database
     (RPC)              (Rust)           (Fast Lookups)
```

- **Input**: Ethereum blocks via JSON-RPC
- **Processing**: Transaction filtering and data transformation
- **Output**: TigerBeetle transfers and accounts for high-performance queries

## 📋 Prerequisites

- **Rust** (latest stable version)
- **TigerBeetle** server running locally
- **Ethereum RPC endpoint** (public or private)

## 🛠️ Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/amiabix/zkCoprocessor.git
   cd zkCoprocessor
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Set up TigerBeetle** (if not already running):
   ```bash
   # Format TigerBeetle database
   tigerbeetle format --cluster=0 --replica=0 --replica-count=1 0_0.tigerbeetle
   
   # Start TigerBeetle server
   tigerbeetle start --addresses=3000 0_0.tigerbeetle
   ```

## 🎯 Usage

### Basic Commands

```bash
# Test TigerBeetle connection
cargo run -- test-tiger

# Test Ethereum RPC connection
cargo run -- test-eth

# Sync Ethereum blocks to TigerBeetle
cargo run -- sync-blocks --from 19000000 --to 19000001

# Run performance benchmarks
cargo run -- benchmark --num-transactions 50

# Debug stored transfers
cargo run -- debug --limit 10

# Query specific data
cargo run -- query --account-id 123456789
cargo run -- query --transfer-id 19000000000000
```

### Command Reference

#### `test-tiger`
Tests connection to TigerBeetle server at `127.0.0.1:3000`

#### `test-eth [--rpc-url <URL>]`
Tests connection to Ethereum RPC endpoint
- Default RPC URL: `https://eth.llamarpc.com`

#### `sync-blocks --from <BLOCK> --to <BLOCK> [--rpc-url <URL>]`
Synchronizes Ethereum blocks to TigerBeetle
- Only processes transactions with value > 0
- Creates accounts for sender and receiver addresses
- Stores transfers with block number in user_data_128

#### `benchmark [--num-transactions <N>] [--include-ethereum] [--rpc-url <URL>]`
Runs performance benchmarks comparing TigerBeetle vs Ethereum RPC
- Default: 50 transactions
- Use `--include-ethereum` to also test against Ethereum RPC (slower)

#### `debug [--limit <N>]`
Inspects stored transfers and accounts in TigerBeetle
- Shows transfer details: amount, accounts, block number
- Default limit: 10 items

#### `query [--account-id <ID>] [--transfer-id <ID>]`
Queries specific accounts or transfers by ID

## 📊 Performance

### Benchmark Results Example
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

## 🔧 Configuration

### Environment Variables
- `TIGERBEETLE_ADDRESS`: TigerBeetle server address (default: `127.0.0.1:3000`)
- `ETH_RPC_URL`: Ethereum RPC endpoint (default: `https://eth.llamarpc.com`)

### TigerBeetle Setup
The application expects TigerBeetle to be running on port 3000. You can modify the connection in `src/main.rs` if needed.

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

## 📞 Support

For issues and questions:
- Create an issue on GitHub
- Check the troubleshooting section above
- Review the debug output for error details

---

**zkCoprocessor** - Bridging Ethereum and TigerBeetle for zero-knowledge applications 🚀 