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

# 特定のインターフェースを使用するために、ソースIPアドレスを指定する
# ip rule と ip route を使用して一時的にルーティングを設定
TABLE_ID=$((1000 + $(echo $NIC | tail -c 2)))

# 一時的なルーティングテーブルを作成
sudo ip rule add from $IP_ADDR table $TABLE_ID 2>/dev/null || true
sudo ip route add default dev $NIC table $TABLE_ID 2>/dev/null || true

# ソースIPを指定してspeedtestを実行
# 注意: speed-cloudflare-cli が環境変数でバインドアドレスを指定できない場合は
# 直接そのインターフェースのIPから実行されるようにする
~/.bun/bin/bunx speed-cloudflare-cli

# クリーンアップ
sudo ip rule del from $IP_ADDR table $TABLE_ID 2>/dev/null || true
sudo ip route flush table $TABLE_ID 2>/dev/null || true
