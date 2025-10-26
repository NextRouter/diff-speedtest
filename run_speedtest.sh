#!/bin/bash

# 引数: nic (eth0 または eth1)
NIC=$1

if [ -z "$NIC" ]; then
    echo "Usage: $0 <interface>" >&2
    exit 1
fi

# 指定されたインターフェースでスピードテストを実行
# インターフェースのIPアドレスを取得
IP_ADDR=$(ip -4 addr show "$NIC" 2>/dev/null | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -1)

if [ -z "$IP_ADDR" ]; then
    echo "Error: Could not find IP address for interface $NIC" >&2
    exit 1
fi

# LD_PRELOAD を使ってソケットを特定のインターフェースにバインドする方法もありますが、
# 最も確実な方法は、network namespace を使用するか、
# または単純に各NICのゲートウェイを通じてルーティングする方法です

# 一時的なルーティングテーブル番号を生成 (NIC名から)
if [ "$NIC" = "eth0" ]; then
    TABLE_ID=100
elif [ "$NIC" = "eth1" ]; then
    TABLE_ID=101
else
    TABLE_ID=102
fi

# デフォルトゲートウェイを取得
GATEWAY=$(ip route show dev $NIC | grep -oP '(?<=via )\S+' | head -1)

if [ -z "$GATEWAY" ]; then
    # ゲートウェイが見つからない場合は、直接接続されたネットワークを使用
    GATEWAY=$(ip route show dev $NIC | grep -oP '^\S+' | head -1)
fi

# 特定のインターフェース経由でコマンドを実行
# sudo権限が必要な場合は、事前に sudoers を設定しておく
sudo ip rule add from $IP_ADDR table $TABLE_ID 2>/dev/null || true
if [ -n "$GATEWAY" ]; then
    sudo ip route add default via $GATEWAY dev $NIC table $TABLE_ID 2>/dev/null || true
else
    sudo ip route add default dev $NIC table $TABLE_ID 2>/dev/null || true
fi
sudo ip route flush cache 2>/dev/null || true

# スピードテストを実行
OUTPUT=$(~/.bun/bin/bunx speed-cloudflare-cli 2>&1)
EXIT_CODE=$?

# ルーティングルールをクリーンアップ
sudo ip rule del from $IP_ADDR table $TABLE_ID 2>/dev/null || true
sudo ip route flush table $TABLE_ID 2>/dev/null || true
sudo ip route flush cache 2>/dev/null || true

# 結果を出力
echo "$OUTPUT"
exit $EXIT_CODE
