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
    echo -e "${RED}Error: speedtest is not installed${NC}"
    echo -e "${YELLOW}Please install speedtest-cli from Ookla:${NC}"
    echo "  curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | sudo bash"
    echo "  sudo apt-get install speedtest"
    exit 1
fi

# プログラムのビルド
echo -e "${YELLOW}Building the project...${NC}"
cargo build --release

# プログラムの実行
echo -e "${GREEN}Running diff-speedtest...${NC}"
sudo ./target/release/diff-speedtest

echo -e "${GREEN}=== Completed ===${NC}"
