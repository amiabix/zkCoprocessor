# zkCoprocessor

A simple Rust demo that syncs Ethereum blockchain data to TigerBeetle. This project shows how to fetch Ethereum blocks via RPC and store transactions as TigerBeetle transfers for fast lookups.

## What It Does

- Fetches Ethereum blocks and transactions via JSON-RPC
- Stores transactions with value > 0 as TigerBeetle transfers
- Provides basic CLI commands for testing and debugging
- Includes simple performance benchmarks

## Prerequisites

- Rust (latest stable)
- TigerBeetle server
- Ethereum RPC endpoint

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

### Benchmarks
```bash
# Run performance tests
cargo run -- benchmark [--num-transactions <N>] [--include-ethereum]
```

## How It Works

1. **Ethereum RPC**: Fetches blocks and transactions from Ethereum
2. **Filtering**: Only processes transactions with value > 0
3. **TigerBeetle Storage**: Creates accounts and transfers in TigerBeetle
4. **ID Mapping**: Converts Ethereum addresses to u128 IDs for TigerBeetle

## Project Structure

```
src/
├── main.rs          # Main CLI and sync logic
└── benchmark.rs     # Performance benchmarking
```

## Configuration

- **TigerBeetle**: `127.0.0.1:3000` (default)
- **Ethereum RPC**: `https://eth.llamarpc.com` (default)

## Troubleshooting

- **No transfers found**: Run sync first, then debug
- **Connection errors**: Check if TigerBeetle is running
- **RPC errors**: Try a different Ethereum RPC endpoint

---

A simple demo of Ethereum → TigerBeetle integration 🚀 