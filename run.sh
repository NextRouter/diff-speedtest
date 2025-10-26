#!/bin/bash

# ビルドと実行
set -e

echo "Building Rust project..."
cargo build --release

echo ""
echo "Setting execute permissions on run_speedtest.sh..."
chmod +x run_speedtest.sh

echo ""
echo "Running the application..."
./target/release/diff-speedtest
