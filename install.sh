#!/bin/bash
set -e

echo "📦 Spear Launcher Installer"
echo "=========================="

# Check if cargo is installed on the host
if ! command -v cargo &> /dev/null; then
    # If cargo is missing, check if toolbox is available (standard on Fedora Silverblue)
    if command -v toolbox &> /dev/null; then
        echo "🔧 'cargo' not found on the host, but 'toolbox' is available."
        echo "   Attempting to compile and install inside the 'main' container..."
        toolbox run --container main cargo run --bin install
        exit 0
    else
        echo "❌ Error: 'cargo' is not installed and 'toolbox' is not available."
        echo "   Please install Rust/Cargo (https://rustup.rs/) and try again."
        exit 1
    fi
fi

# Run natively if cargo is available
echo "🚀 Compiling and installing Spear natively..."
cargo run --bin install
