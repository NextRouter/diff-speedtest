# Diff Speedtest

Ubuntu 上で speedtest を実行し、Prometheus の TCP トラフィックデータと比較して、その比率を外部エンドポイントに送信するツールです。

## 機能

- eth0 と eth1 の 2 つのインターフェースで speedtest を実行
- 各インターフェースの Download 速度を取得
- Prometheus から TCP トラフィックの帯域幅データを取得
- `Download速度(bps) / TCP帯域幅(bps)` の比率を計算
- 計算結果を指定エンドポイントに送信

## 必要な環境

- Ubuntu (Linux)
- Rust (1.70 以降推奨)
- speedtest-cli (Ookla 版)
- Prometheus (localhost:9090 で稼働)
- 送信先エンドポイント (localhost:32600 で稼働)

## インストール

### 1. Speedtest CLI のインストール

```bash
curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | sudo bash
sudo apt-get install speedtest
```

### 2. プロジェクトのビルド

```bash
cargo build --release
```

## 使用方法

### シェルスクリプトで実行（推奨）

```bash
chmod +x run.sh
./run.sh
```

### 直接実行

```bash
sudo ./target/release/diff-speedtest
```

※ speedtest はネットワークインターフェースを指定するため、sudo 権限が必要な場合があります。

## 設定

### インターフェースの変更

`src/main.rs`の`main`関数内で、インターフェース設定を変更できます：

```rust
let interfaces = vec![
    InterfaceConfig {
        nic_name: "eth0".to_string(),
        wan_name: "wan0".to_string(),
    },
    InterfaceConfig {
        nic_name: "eth1".to_string(),
        wan_name: "wan1".to_string(),
    },
];
```

### Speedtest サーバーの変更

`src/main.rs`の`run_speedtest`関数内で、サーバー ID を変更できます：

```rust
.args(&["-s", "48463", "-I", interface])  // 48463を別のサーバーIDに変更
```

## 動作フロー

1. **eth0 で speedtest 実行**

   - `speedtest -s 48463 -I eth0`を実行
   - Download 速度を抽出（例: 640.00 Mbps）

2. **Prometheus から eth0 のデータ取得**

   - クエリ: `tcp_traffic_scan_tcp_bandwidth_avg_bps{interface="eth0"}`
   - 値を取得（例: 20779245.419303708 bps）

3. **比率を計算**

   - `640.00 * 1000000 / 20779245.419303708` = 約 30.8

4. **エンドポイントに送信**

   - `curl "http://localhost:32600/tcpflow?value=30.8&nic=wan0"`

5. **eth1 についても同様の処理を実行**

## トラブルシューティング

### speedtest が見つからない

```bash
speedtest --version
```

で確認し、インストールされていない場合は上記のインストール手順を実行してください。

### Prometheus にデータがない

Prometheus が正しく稼働しているか、また`tcp_traffic_scan_tcp_bandwidth_avg_bps`メトリクスが収集されているか確認してください：

```bash
curl 'http://localhost:9090/api/v1/query?query=tcp_traffic_scan_tcp_bandwidth_avg_bps'
```

### エンドポイントへの送信が失敗する

エンドポイントが稼働しているか確認してください：

```bash
curl http://localhost:32600/health
```

## ライセンス

MIT
