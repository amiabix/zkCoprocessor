#!/bin/bash

# zkCoprocessor AWS Deployment Script
# This script sets up the complete environment on an Ubuntu EC2 instance

set -e  # Exit on any error

echo "🚀 zkCoprocessor AWS Deployment Script"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root. Please run as ubuntu user."
   exit 1
fi

# Update system
print_status "Updating system packages..."
sudo apt update && sudo apt upgrade -y
print_success "System updated"

# Install essential packages
print_status "Installing essential packages..."
sudo apt install -y build-essential pkg-config libssl-dev curl git
print_success "Essential packages installed"

# Install Rust
print_status "Installing Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_success "Rust installed"
else
    print_warning "Rust already installed"
fi

# Install TigerBeetle
print_status "Installing TigerBeetle..."
if ! command -v tigerbeetle &> /dev/null; then
    TIGERBEETLE_VERSION="0.16.45"
    TIGERBEETLE_URL="https://github.com/tigerbeetle/tigerbeetle/releases/latest/download/tigerbeetle_$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/x86_64/amd64/')_${TIGERBEETLE_VERSION}.tar.gz"
    
    curl -L $TIGERBEETLE_URL | tar -xz
    sudo mv tigerbeetle /usr/local/bin/
    print_success "TigerBeetle installed"
else
    print_warning "TigerBeetle already installed"
fi

# Install ZisK
print_status "Installing ZisK..."
if ! command -v zisk &> /dev/null; then
    ZISK_URL="https://github.com/delendum-xyz/zisk/releases/latest/download/zisk-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m | sed 's/x86_64/x64/')"
    
    curl -L $ZISK_URL -o zisk
    chmod +x zisk
    sudo mv zisk /usr/local/bin/
    print_success "ZisK installed"
else
    print_warning "ZisK already installed"
fi

# Clone repository
print_status "Cloning zkCoprocessor repository..."
if [ ! -d "zkCoprocessor" ]; then
    git clone https://github.com/amiabix/zkCoprocessor.git
    cd zkCoprocessor
    print_success "Repository cloned"
else
    cd zkCoprocessor
    print_warning "Repository already exists, updating..."
    git pull origin main
    print_success "Repository updated"
fi

# Build project
print_status "Building zkCoprocessor..."
cargo build --release
print_success "Project built successfully"

# Set up TigerBeetle database
print_status "Setting up TigerBeetle database..."
if [ ! -f "0_0.tigerbeetle" ]; then
    tigerbeetle format --cluster=0 --replica=0 --replica-count=1 0_0.tigerbeetle
    print_success "TigerBeetle database formatted"
else
    print_warning "TigerBeetle database already exists"
fi

# Check if TigerBeetle is running
if ! pgrep -f "tigerbeetle start" > /dev/null; then
    print_status "Starting TigerBeetle server..."
    nohup tigerbeetle start --addresses=3000 0_0.tigerbeetle > tigerbeetle.log 2>&1 &
    sleep 3  # Give it time to start
    print_success "TigerBeetle server started"
else
    print_warning "TigerBeetle server already running"
fi

# Test the setup
print_status "Testing the setup..."

# Test TigerBeetle connection
if ./target/release/zkcoprocessor test-tiger; then
    print_success "TigerBeetle connection test passed"
else
    print_error "TigerBeetle connection test failed"
    exit 1
fi

# Test Ethereum RPC
if ./target/release/zkcoprocessor test-eth; then
    print_success "Ethereum RPC test passed"
else
    print_warning "Ethereum RPC test failed (this is normal if no RPC endpoint is configured)"
fi

# Test ZisK
if command -v zisk &> /dev/null; then
    print_success "ZisK is available for real ZK proofs"
else
    print_error "ZisK not found - ZK proofs will use simulation mode"
fi

# Create systemd service for TigerBeetle (optional)
read -p "Do you want to create a systemd service for TigerBeetle? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Creating systemd service for TigerBeetle..."
    
    sudo tee /etc/systemd/system/tigerbeetle.service > /dev/null <<EOF
[Unit]
Description=TigerBeetle Database Server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$(pwd)
ExecStart=/usr/local/bin/tigerbeetle start --addresses=3000 0_0.tigerbeetle
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable tigerbeetle
    print_success "Systemd service created and enabled"
fi

# Final status
echo
echo "🎉 Deployment Complete!"
echo "======================"
echo
echo "📁 Project location: $(pwd)"
echo "🔧 TigerBeetle: $(which tigerbeetle)"
echo "🔐 ZisK: $(which zisk)"
echo "🚀 Binary: ./target/release/zkcoprocessor"
echo
echo "🧪 Test commands:"
echo "  ./target/release/zkcoprocessor test-tiger"
echo "  ./target/release/zkcoprocessor test-eth"
echo "  ./target/release/zkcoprocessor sync-blocks --from 19000000 --to 19000000"
echo "  ./target/release/zkcoprocessor prove-transaction --transfer-id 19000000000000"
echo
echo "📊 Monitor TigerBeetle:"
echo "  tail -f tigerbeetle.log"
echo "  sudo systemctl status tigerbeetle  # if using systemd"
echo
echo "🔧 Useful commands:"
echo "  cargo build --release              # Rebuild"
echo "  cargo run -- --help                # Show all commands"
echo "  ./target/release/zkcoprocessor debug --limit 5  # View stored data"
echo
print_success "zkCoprocessor is ready for production use!" 