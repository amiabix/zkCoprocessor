#!/bin/bash

# ZisK Setup Script for zkCoprocessor
# This script will help set up ZisK when macOS support is available

set -e

echo "🚀 Setting up ZisK for zkCoprocessor..."

# Check if we're on a supported platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "⚠️  macOS detected - ZisK support is not yet available"
    echo "   The system will use simulation mode for development"
    echo "   Check https://github.com/0xPolygonHermez/zisk for updates"
    exit 0
fi

# Check if ZisK is already installed
if command -v cargo-zisk &> /dev/null; then
    echo "✅ ZisK is already installed"
    cargo-zisk --version
    exit 0
fi

echo "📦 Installing ZisK..."

# Clone ZisK repository
if [ ! -d "zisk" ]; then
    echo "📥 Cloning ZisK repository..."
    git clone https://github.com/0xPolygonHermez/zisk.git
fi

cd zisk

# Build and install cargo-zisk
echo "🔨 Building cargo-zisk..."
cd cli
cargo build --release
cargo install --path .

echo "✅ ZisK installation completed!"
echo "   Run 'cargo-zisk --version' to verify installation"

# Go back to project root
cd ../..

echo "🎉 ZisK is ready for zkCoprocessor!"
echo "   You can now use real ZK proof generation with:"
echo "   cargo run -- prove-transaction --transfer-id <ID>" 