# diff-speedtest

Ubuntu 上で NIC ごとにスピードテストを実行し、Prometheus データと比較して API に送信する Rust プログラムです。

## 機能

- 各 NIC（eth0, eth1）で Cloudflare スピードテストを実行
- Download speed を抽出
- Prometheus から対応する TCP トラフィックデータを取得
- `Download speed (bps) / TCP bandwidth (bps)` を計算
- 計算結果を API エンドポイントに送信

## 前提条件

Ubuntu 環境で以下がインストールされている必要があります：

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Bun (bunxコマンドを使用)
curl -fsSL https://bun.sh/install | bash

# iproute2 (ipコマンド)
sudo apt install iproute2
```

## 使い方

### ビルドと実行

```bash
chmod +x run.sh
./run.sh
```

または手動で：

```bash
# ビルド
cargo build --release

# 実行権限の付与
chmod +x run_speedtest.sh

# 実行
./target/release/diff-speedtest
```

## 設定

### NIC マッピング

`src/main.rs` の `main()` 関数内で設定：

```rust
let nics = vec![("eth0", "wan0"), ("eth1", "wan1")];
```

- `eth0` → `wan0`
- `eth1` → `wan1`

### エンドポイント

- **Prometheus**: `http://localhost:9090`
- **API**: `http://localhost:32600/tcpflow?value={値}&nic={nic}`

## 処理フロー

1. **スピードテスト実行**: `run_speedtest.sh` が各 NIC で `bunx speed-cloudflare-cli` を実行
2. **データ抽出**: Download speed の値をパース（Mbps）
3. **Prometheus クエリ**:
   ```
   {job="tcp-traffic-scan",__name__=~"tcp_traffic_scan_tcp_bandwidth_avg_bps",interface="<nic>"}
   ```
4. **計算**: `Download speed (Mbps) × 1,000,000 / TCP bandwidth (bps)`
5. **API 送信**: `GET http://localhost:32600/tcpflow?value={ratio}&nic={wan_name}`

## トラブルシューティング

### bunx コマンドが見つからない場合

```bash
# Bunのインストール確認
which bunx

# パスを確認してrun_speedtest.shを編集
# /path/to/bunx speed-cloudflare-cli
```

### インターフェースが見つからない場合

```bash
# 利用可能なインターフェースを確認
ip link show
```

### Prometheus に接続できない場合

```bash
# Prometheusが起動しているか確認
curl http://localhost:9090/api/v1/query?query=up
```
