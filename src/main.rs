use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct PrometheusResponse {
    data: PrometheusData,
}

#[derive(Debug, Deserialize)]
struct PrometheusData {
    result: Vec<PrometheusResult>,
}

#[derive(Debug, Deserialize)]
struct PrometheusResult {
    metric: PrometheusMetric,
    value: (f64, String),
}

#[derive(Debug, Deserialize)]
struct PrometheusMetric {
    interface: String,
}

struct InterfaceConfig {
    nic_name: String,
    wan_name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
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

    for interface in interfaces {
        process_interface(&interface).await?;
    }

    Ok(())
}

async fn process_interface(config: &InterfaceConfig) -> Result<()> {
    println!("Processing interface: {}", config.nic_name);

    // 1. speedtestを実行
    let download_mbps = run_speedtest(&config.nic_name)?;
    println!(
        "  Download speed for {}: {} Mbps",
        config.nic_name, download_mbps
    );

    // 2. Prometheusからデータ取得
    let tcp_bandwidth = get_prometheus_bandwidth(&config.nic_name).await?;
    println!(
        "  TCP bandwidth for {}: {} bps",
        config.nic_name, tcp_bandwidth
    );

    // 3. 計算
    let download_bps = download_mbps * 1_000_000.0; // Mbpsをbpsに変換
    let ratio = download_bps / tcp_bandwidth;
    println!("  Ratio for {}: {}", config.nic_name, ratio);

    // 4. curlで送信
    send_to_endpoint(&config.wan_name, ratio).await?;
    println!("  Sent to endpoint for {}: {}", config.wan_name, ratio);

    Ok(())
}

fn run_speedtest(interface: &str) -> Result<f64> {
    println!("  Running: speedtest -s 48463 -I {}", interface);

    let output = Command::new("speedtest")
        .args(&["--accept-license", "-s", "48463", "-I", interface])
        .output()
        .context("Failed to execute speedtest")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // デバッグ用に出力を表示
    if !stdout.is_empty() {
        println!("  STDOUT:\n{}", stdout);
    }
    if !stderr.is_empty() {
        println!("  STDERR:\n{}", stderr);
    }

    if !output.status.success() {
        anyhow::bail!(
            "Speedtest failed with exit code {:?}\nSTDOUT: {}\nSTDERR: {}",
            output.status.code(),
            stdout,
            stderr
        );
    }

    // Downloadの値を抽出
    let re = Regex::new(r"Download:\s+([0-9.]+)\s+Mbps")?;
    let captures = re
        .captures(&stdout)
        .context("Could not find Download speed in speedtest output")?;

    let download_mbps: f64 = captures
        .get(1)
        .context("Could not extract Download speed value")?
        .as_str()
        .parse()
        .context("Could not parse Download speed as float")?;

    Ok(download_mbps)
}

async fn get_prometheus_bandwidth(interface: &str) -> Result<f64> {
    let query = format!(
        r#"tcp_traffic_scan_tcp_bandwidth_avg_bps{{interface="{}"}}"#,
        interface
    );
    let url = format!("http://localhost:9090/api/v1/query?query={}", query);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to query Prometheus")?;

    let prom_response: PrometheusResponse = response
        .json()
        .await
        .context("Failed to parse Prometheus response")?;

    if prom_response.data.result.is_empty() {
        anyhow::bail!("No data found in Prometheus for interface {}", interface);
    }

    let bandwidth: f64 = prom_response.data.result[0]
        .value
        .1
        .parse()
        .context("Could not parse bandwidth value")?;

    Ok(bandwidth)
}

async fn send_to_endpoint(wan_name: &str, value: f64) -> Result<()> {
    let url = format!(
        "http://localhost:32600/tcpflow?value={}&nic={}",
        value, wan_name
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to send to endpoint")?;

    if !response.status().is_success() {
        anyhow::bail!("Endpoint returned error: {}", response.status());
    }

    Ok(())
}
