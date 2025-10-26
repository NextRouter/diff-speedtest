#!/bin/bash

# 引数: nic (eth0 または eth1)
NIC=$1

if [ -z "$NIC" ]; then
    echo "Usage: $0 <interface>" >&2
    exit 1
fi

# インターフェースのIPアドレスを取得
IP_ADDR=$(ip -4 addr show "$NIC" | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -1)

if [ -z "$IP_ADDR" ]; then
    echo "Error: Could not find IP address for interface $NIC" >&2
    exit 1
fi

# ゲートウェイを取得
GATEWAY=$(ip route | grep "default.*$NIC" | awk '{print $3}' | head -1)

if [ -z "$GATEWAY" ]; then
    echo "Error: Could not find gateway for interface $NIC" >&2
    exit 1
fi

# 一意のNetwork Namespace名
NETNS="speedtest_${NIC}_$$"

# クリーンアップ関数
cleanup() {
    # vethペアを削除
    sudo ip link delete "veth_${NIC}" 2>/dev/null || true
    # namespaceを削除
    sudo ip netns delete "$NETNS" 2>/dev/null || true
}
trap cleanup EXIT

# Network Namespaceを作成
sudo ip netns add "$NETNS"

# vethペアを作成（ホスト側: veth_${NIC}, namespace側: veth0）
sudo ip link add "veth_${NIC}" type veth peer name veth0

# veth0をnamespaceに移動
sudo ip link set veth0 netns "$NETNS"

# ホスト側のvethを起動
sudo ip link set "veth_${NIC}" up

# Namespace内のインターフェースを設定
sudo ip netns exec "$NETNS" ip link set lo up
sudo ip netns exec "$NETNS" ip link set veth0 up
sudo ip netns exec "$NETNS" ip addr add 10.200.1.1/30 dev veth0
sudo ip netns exec "$NETNS" ip route add default via 10.200.1.2

# ホスト側のvethにIPを設定
sudo ip addr add 10.200.1.2/30 dev "veth_${NIC}"

# IP forwardingを有効化
sudo sysctl -w net.ipv4.ip_forward=1 > /dev/null 2>&1

# iptablesでNATを設定（namespace -> 特定のNIC経由でインターネット）
sudo iptables -t nat -A POSTROUTING -s 10.200.1.0/30 -o "$NIC" -j MASQUERADE
sudo iptables -A FORWARD -i "veth_${NIC}" -o "$NIC" -j ACCEPT
sudo iptables -A FORWARD -i "$NIC" -o "veth_${NIC}" -m state --state RELATED,ESTABLISHED -j ACCEPT

# Network Namespace内でスピードテストを実行
sudo ip netns exec "$NETNS" su - user -c "cd /home/user/diff-speedtest && /home/user/.bun/bin/bunx speed-cloudflare-cli"

# クリーンアップ（iptablesルールも削除）
sudo iptables -t nat -D POSTROUTING -s 10.200.1.0/30 -o "$NIC" -j MASQUERADE 2>/dev/null || true
sudo iptables -D FORWARD -i "veth_${NIC}" -o "$NIC" -j ACCEPT 2>/dev/null || true
sudo iptables -D FORWARD -i "$NIC" -o "veth_${NIC}" -m state --state RELATED,ESTABLISHED -j ACCEPT 2>/dev/null || true

cleanup
trap - EXIT



