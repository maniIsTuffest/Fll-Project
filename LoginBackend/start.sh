#!/bin/bash

# Login Backend Startup Script for Unix/Linux/macOS

echo ""
echo "===================================="
echo "  Login Backend System Startup"
echo "===================================="
echo ""

# Check if we're in the right directory
if [ ! -f "main/Cargo.toml" ]; then
    echo "Error: This script must be run from the LoginBackend directory"
    echo "Current directory: $(pwd)"
    exit 1
fi

if [ ! -f "api/api/Gemfile" ]; then
    echo "Error: Could not find api/api/Gemfile"
    exit 1
fi

echo "Checking prerequisites..."
echo ""

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust is not installed"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi
echo "[OK] Rust found: $(rustc --version)"

# Check if Ruby is installed
if ! command -v ruby &> /dev/null; then
    echo "Error: Ruby is not installed"
    echo "Please install Ruby from https://www.ruby-lang.org/en/downloads/"
    exit 1
fi
echo "[OK] Ruby found: $(ruby --version)"

# Check if Bundler is installed
if ! command -v bundle &> /dev/null; then
    echo "Error: Bundler is not installed"
    echo "Please run: gem install bundler"
    exit 1
fi
echo "[OK] Bundler found: $(bundle --version)"

echo ""
echo "===================================="
echo "  Installing Dependencies"
echo "===================================="
echo ""

# Install Rails dependencies
echo "Installing Rails dependencies..."
cd api/api
bundle install
if [ $? -ne 0 ]; then
    echo "Error: Failed to install Rails dependencies"
    cd ../..
    exit 1
fi
cd ../..
echo "[OK] Rails dependencies installed"

echo ""
echo "===================================="
echo "  Starting Services"
echo "===================================="
echo ""
echo "Rust API Server:  http://localhost:8000"
echo "Rails API:        http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop all services"
echo ""

# Start the Rust backend (which will start Rails as a subprocess)
cd main
cargo run

cd ..
