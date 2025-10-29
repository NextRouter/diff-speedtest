#!/bin/bash

# Ubuntu上でdiff-speedtestを実行するシェルスクリプト

set -e

# カラー出力設定
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Diff Speedtest Runner ===${NC}"

# speedtestのインストール確認
if ! command -v speedtest &> /dev/null; then
    sudo apt-get install curl
    curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | sudo bash
    sudo apt-get install speedtest
fi

# プログラムのビルド
echo -e "${YELLOW}Building the project...${NC}"
cargo build --release

# プログラムの実行
echo -e "${GREEN}Running diff-speedtest...${NC}"

curl "http://localhost:32600/tcpflow?value=1&nic=eth0"
curl "http://localhost:32600/tcpflow?value=1&nic=eth1"

sleep 1

# sudoが必要かどうかを確認
if [ "$EUID" -ne 0 ]; then
    echo -e "${YELLOW}Note: Running with sudo for network interface access${NC}"
    sudo ./target/release/diff-speedtest
else
    ./target/release/diff-speedtest
fi

echo -e "${GREEN}=== Completed ===${NC}"
