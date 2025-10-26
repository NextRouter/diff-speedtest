#!/bin/bash

# ビルドと実行
set -e

echo "Building Rust project..."
cargo build --release

sudo apt install unzip -y
curl -fsSL https://bun.com/install | bash
source /home/user/.bashrc

echo ""
echo "Setting execute permissions on run_speedtest.sh..."
chmod +x run_speedtest.sh

echo ""
echo "Running the application..."
./target/release/diff-speedtest