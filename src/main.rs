use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct PrometheusResponse {
    status: String,
    data: PrometheusData,
}

#[derive(Debug, Deserialize)]
struct PrometheusData {
    result: Vec<PrometheusResult>,
}

#[derive(Debug, Deserialize)]
struct PrometheusResult {
    #[allow(dead_code)]
    metric: PrometheusMetric,
    value: (f64, String),
}

#[derive(Debug, Deserialize)]
struct PrometheusMetric {
    #[allow(dead_code)]
    interface: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // NICの設定: (interface, wan_name)
    let nics = vec![("eth0", "wan0"), ("eth1", "wan1")];

    for (interface, wan_name) in nics {
        println!("Processing interface: {} ({})", interface, wan_name);

        // エラーが発生しても次のNICの処理を続ける
        let result = process_interface(interface, wan_name).await;

        match result {
            Ok(_) => println!("  Successfully completed for {}\n", interface),
            Err(e) => {
                eprintln!("  Error processing {}: {}\n", interface, e);
                continue;
            }
        }
    }

    Ok(())
}

async fn process_interface(interface: &str, wan_name: &str) -> Result<()> {
    // 1. スピードテストを実行してDownload speedを取得
    let download_speed = run_speedtest(interface.to_string())
        .await
        .context(format!("Failed to run speedtest for {}", interface))?;

    println!("  Download speed: {:.2} Mbps", download_speed);

    // 2. Prometheusからトラフィックデータを取得
    let tcp_bandwidth = query_prometheus(interface)
        .await
        .context(format!("Failed to query Prometheus for {}", interface))?;

    println!("  TCP bandwidth: {:.2} bps", tcp_bandwidth);

    // 3. 計算: Download speed (Mbps) / TCP bandwidth (bps)
    // Download speedをMbpsからbpsに変換
    let download_speed_bps = download_speed * 1_000_000.0;
    let ratio = download_speed_bps / tcp_bandwidth;

    println!("  Calculated ratio: {:.6}", ratio);

    // 4. APIに送信
    send_to_api(ratio, wan_name)
        .await
        .context(format!("Failed to send to API for {}", wan_name))?;

    println!("  Successfully sent to API");

    Ok(())
}

async fn run_speedtest(interface: String) -> Result<f64> {
    // シェルスクリプトを使用してスピードテストを実行
    let output = Command::new("./run_speedtest.sh")
        .arg(interface)
        .output()
        .context("Failed to execute speedtest script")?;

    if !output.status.success() {
        anyhow::bail!(
            "Speedtest failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // "Download speed: XXX.XX Mbps" を抽出（複数スペースに対応）
    let re = Regex::new(r"Download\s+speed:\s+([\d.]+)\s+Mbps")?;

    if let Some(caps) = re.captures(&stdout) {
        let speed = caps[1].parse::<f64>()?;
        Ok(speed)
    } else {
        anyhow::bail!("Could not parse download speed from output: {}", stdout);
    }
}

async fn query_prometheus(interface: &str) -> Result<f64> {
    let query = format!(
        r#"{{job="tcp-traffic-scan",__name__=~"tcp_traffic_scan_tcp_bandwidth_avg_bps",interface="{}"}}"#,
        interface
    );

    let url = format!(
        "http://localhost:9090/api/v1/query?query={}",
        urlencoding::encode(&query)
    );

    let client = reqwest::Client::new();
    let response: PrometheusResponse = client.get(&url).send().await?.json().await?;

    if response.status != "success" {
        anyhow::bail!("Prometheus query failed");
    }

    if response.data.result.is_empty() {
        anyhow::bail!("No data found for interface {}", interface);
    }

    // 最初の結果を使用
    let value = response.data.result[0].value.1.parse::<f64>()?;

    Ok(value)
}

async fn send_to_api(value: f64, nic: &str) -> Result<()> {
    let url = format!("http://localhost:32600/tcpflow?value={}&nic={}", value, nic);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!("API request failed with status: {}", response.status());
    }

    Ok(())
}
