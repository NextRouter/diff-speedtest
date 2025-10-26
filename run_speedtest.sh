#!/bin/bash

# 引数: nic (eth0 または eth1)
NIC=$1

if [ -z "$NIC" ]; then
    echo "Usage: $0 <interface>" >&2
    exit 1
fi

# 指定されたインターフェースでスピードテストを実行
# --bind-address オプションでインターフェースを指定
# インターフェースのIPアドレスを取得
IP_ADDR=$(ip -4 addr show "$NIC" | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -1)

if [ -z "$IP_ADDR" ]; then
    echo "Error: Could not find IP address for interface $NIC" >&2
    exit 1
fi

# bunxコマンドを実行（環境によってはbunxのパスを指定する必要があります）
# --bind-address でIPを指定して特定のインターフェースから接続
~/.bun/bin/bunx speed-cloudflare-cli
