#!/bin/bash

# ビルドと実行
set -e

echo "Building Rust project..."
cargo build --release

sudo su

if [ -f /home/user/.bun/bin/bunx ]; then
    echo "bunx is already installed."
else
    echo "bunx is not installed. Installing bun..."
    sudo apt install unzip -y
    curl -fsSL https://bun.sh/install | bash
    source ~/.bashrc
fi

echo ""
echo "Setting execute permissions on run_speedtest.sh..."
chmod +x run_speedtest.sh

echo ""
echo "Running the application..."
./target/release/diff-speedtest