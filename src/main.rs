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
    #[allow(dead_code)]
    metric: PrometheusMetric,
    value: (f64, String),
}

#[derive(Debug, Deserialize)]
struct PrometheusMetric {
    #[allow(dead_code)]
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

    println!("Starting speedtest for all interfaces in parallel...\n");

    // Prometheusからデータを並列取得
    let prometheus_tasks: Vec<_> = interfaces
        .iter()
        .map(|config| {
            let nic_name = config.nic_name.clone();
            tokio::spawn(async move { get_prometheus_bandwidth(&nic_name).await })
        })
        .collect();

    let mut tcp_bandwidth_list = Vec::new();
    for (i, task) in prometheus_tasks.into_iter().enumerate() {
        let bandwidth = task.await.context("Prometheus task panicked")??;
        tcp_bandwidth_list.push(bandwidth);
        println!(
            "  Prometheus data retrieved for {}: {} bps",
            interfaces[i].nic_name, bandwidth
        );
    }

    // すべてのインターフェースのspeedtestを並行実行
    let speedtest_tasks: Vec<_> = interfaces
        .iter()
        .map(|config| {
            let nic_name = config.nic_name.clone();
            tokio::task::spawn_blocking(move || {
                println!("  Starting speedtest for {}", nic_name);
                run_speedtest(&nic_name)
            })
        })
        .collect();

    // すべてのspeedtestの完了を待つ
    let mut speedtest_results = Vec::new();
    for (i, task) in speedtest_tasks.into_iter().enumerate() {
        let result = task.await.context("Speedtest task panicked")??;
        speedtest_results.push(result);
        println!(
            "  ✓ Speedtest completed for {}: {} Mbps",
            interfaces[i].nic_name, result
        );
    }

    println!("\n=== All speedtests completed. Processing results ===\n");

    // 各インターフェースの結果を処理
    for (i, interface) in interfaces.iter().enumerate() {
        println!("--- Processing interface: {} ---", interface.nic_name);

        let download_mbps = speedtest_results[i];
        println!("  Download speed: {} Mbps", download_mbps);

        println!("  TCP bandwidth: {} bps", tcp_bandwidth_list[i]);

        // 計算
        let download_bps = download_mbps * 1_000_000.0; // Mbpsをbpsに変換
        let ratio = download_bps / tcp_bandwidth_list[i];
        println!("  Calculated ratio: {:.4}", ratio);

        // curlで送信
        send_to_endpoint(&interface.wan_name, ratio).await?;
        println!("  ✓ Sent to endpoint for {}\n", interface.wan_name);
    }

    Ok(())
}

fn run_speedtest(interface: &str) -> Result<f64> {
    println!("  → Running speedtest on interface: {}", interface);

    let output = Command::new("speedtest")
        .args(&["--accept-license", "-s", "48463", "-I", interface])
        .output()
        .context(format!("Failed to execute speedtest for {}", interface))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("  ✗ Speedtest failed for {}", interface);
        if !stderr.is_empty() {
            eprintln!("  Error: {}", stderr);
        }
        anyhow::bail!(
            "Speedtest failed for {} with exit code {:?}\nSTDOUT: {}\nSTDERR: {}",
            interface,
            output.status.code(),
            stdout,
            stderr
        );
    }

    // Downloadの値を抽出
    let re = Regex::new(r"Download:\s+([0-9.]+)\s+Mbps")?;
    let captures = re.captures(&stdout).context(format!(
        "Could not find Download speed in speedtest output for {}",
        interface
    ))?;

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
    let response = client.get(&url).send().await.context(format!(
        "Failed to query Prometheus for interface {}",
        interface
    ))?;

    let prom_response: PrometheusResponse = response.json().await.context(format!(
        "Failed to parse Prometheus response for interface {}",
        interface
    ))?;

    if prom_response.data.result.is_empty() {
        anyhow::bail!("No data found in Prometheus for interface {}", interface);
    }

    let bandwidth: f64 = prom_response.data.result[0]
        .value
        .1
        .parse()
        .context(format!(
            "Could not parse bandwidth value for interface {}",
            interface
        ))?;

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
        .context(format!("Failed to send to endpoint for {}", wan_name))?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Endpoint returned error for {}: {}",
            wan_name,
            response.status()
        );
    }

    Ok(())
}
