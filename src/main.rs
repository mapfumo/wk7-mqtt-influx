//! Week 7: MQTT + InfluxDB Gateway Service
//!
//! This service:
//! - Spawns probe-rs as a subprocess to run the Week 5 gateway firmware
//! - Captures stdout and parses JSON telemetry
//! - Publishes telemetry to MQTT broker
//! - Writes telemetry to InfluxDB time-series database
//!
//! Architecture: probe-rs → stdout → parser → channel → processor → MQTT + InfluxDB

pub mod config;
pub mod mqtt;
pub mod influxdb;

use anyhow::{Context, Result};
use config::Config;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Telemetry packet from Node 2 gateway (matches Week 5 JSON format)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TelemetryPacket {
    /// Timestamp in milliseconds since boot
    ts: u32,
    /// Node ID (should be "N2" for gateway)
    id: String,
    /// Node 1 sensor data (remote sensor via LoRa)
    n1: Node1Data,
    /// Node 2 sensor data (gateway local sensor)
    n2: Node2Data,
    /// Signal quality metrics
    sig: SignalQuality,
    /// Statistics
    sts: Statistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node1Data {
    /// Temperature in °C
    t: f32,
    /// Humidity in %
    h: f32,
    /// Gas resistance in ohms
    g: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node2Data {
    /// Temperature in °C (optional, SHT3x may not be reading yet)
    #[serde(skip_serializing_if = "Option::is_none")]
    t: Option<f32>,
    /// Humidity in % (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    h: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignalQuality {
    /// RSSI in dBm
    rssi: i16,
    /// SNR in dB
    snr: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Statistics {
    /// Packets received
    rx: u32,
    /// CRC errors
    err: u32,
}

/// Extract JSON from probe-rs log line
///
/// Example input: `[INFO] JSON sent via VCP: {"ts":12000,...}\n`
/// Returns: `{"ts":12000,...}`
fn extract_json_from_log_line(line: &str) -> Option<String> {
    // Look for the JSON marker in the log
    if let Some(start_idx) = line.find("JSON sent via VCP: ") {
        let json_start = start_idx + "JSON sent via VCP: ".len();
        let json_str = &line[json_start..];

        // Remove the escaped \n and defmt source location suffix
        // Format: {...}\n (wk5_gateway_firmware src/main.rs:573)
        let without_location = json_str
            .split(" (")  // Split on defmt source location
            .next()       // Take everything before the location
            .unwrap_or(json_str)
            .trim();

        // Remove both escaped \\n and actual \n characters
        let json_clean = without_location
            .trim_end_matches("\\n")
            .trim_end_matches('\n')
            .trim();

        Some(json_clean.to_string())
    } else {
        None
    }
}

/// Parse probe-rs stdout and send telemetry packets to channel
async fn parse_probe_rs_output(
    mut reader: BufReader<tokio::process::ChildStdout>,
    tx: mpsc::Sender<TelemetryPacket>,
) -> Result<()> {
    let mut line_buf = String::new();

    info!("Starting probe-rs output parser");

    loop {
        line_buf.clear();

        match reader.read_line(&mut line_buf).await {
            Ok(0) => {
                warn!("probe-rs process ended (EOF on stdout)");
                break;
            }
            Ok(_) => {
                // Try to extract JSON from this line
                if let Some(json_str) = extract_json_from_log_line(&line_buf) {
                    match serde_json::from_str::<TelemetryPacket>(&json_str) {
                        Ok(packet) => {
                            info!(
                                node_id = %packet.id,
                                timestamp_ms = packet.ts,
                                temp_c = packet.n1.t,
                                humidity_pct = packet.n1.h,
                                rssi_dbm = packet.sig.rssi,
                                "Telemetry packet received"
                            );

                            if let Err(e) = tx.send(packet).await {
                                error!(error = %e, "Failed to send packet to channel");
                                break;
                            }
                        }
                        Err(e) => {
                            warn!(error = %e, json = %json_str, "Failed to parse JSON");
                        }
                    }
                } else {
                    // Not a JSON line, just pass through for debugging
                    // (Could filter these to only show important logs)
                    if line_buf.contains("[INFO]") || line_buf.contains("[WARN]") || line_buf.contains("[ERROR]") {
                        print!("{}", line_buf); // Pass through defmt logs
                    }
                }
            }
            Err(e) => {
                error!(error = %e, "Error reading from probe-rs stdout");
                break;
            }
        }
    }

    Ok(())
}

/// Process telemetry packets and publish to MQTT and InfluxDB
async fn process_telemetry(
    mut rx: mpsc::Receiver<TelemetryPacket>,
    mqtt_client: mqtt::MqttClient,
    influxdb_client: influxdb::InfluxDbClient,
    topic_prefix: String,
) {
    info!("Starting telemetry processor");

    while let Some(packet) = rx.recv().await {
        // Log Node 1 (remote sensor) data
        info!(
            timestamp_ms = packet.ts,
            node_id = %packet.id,
            n1_temperature = packet.n1.t,
            n1_humidity = packet.n1.h,
            n1_gas_resistance = packet.n1.g,
            rssi = packet.sig.rssi,
            snr = packet.sig.snr,
            packets_received = packet.sts.rx,
            crc_errors = packet.sts.err,
            "Processing telemetry packet"
        );

        // Log Node 2 (gateway local sensor) data if available
        if packet.n2.t.is_some() || packet.n2.h.is_some() {
            info!(
                n2_temperature = ?packet.n2.t,
                n2_humidity = ?packet.n2.h,
                "Gateway local sensor (SHT3x)"
            );
        }

        // Publish to MQTT (Phase 3)
        if let Err(e) = publish_telemetry_to_mqtt(&mqtt_client, &topic_prefix, &packet).await {
            error!(error = %e, "Failed to publish telemetry to MQTT");
        }

        // Write to InfluxDB (Phase 5)
        if let Err(e) = write_telemetry_to_influxdb(&influxdb_client, &packet).await {
            error!(error = %e, "Failed to write telemetry to InfluxDB");
        }
    }

    info!("Telemetry processor stopped");
}

/// Publish telemetry packet to MQTT topics
async fn publish_telemetry_to_mqtt(
    mqtt_client: &mqtt::MqttClient,
    prefix: &str,
    packet: &TelemetryPacket,
) -> anyhow::Result<()> {
    // Node 1 sensors (remote sensor via LoRa)
    mqtt_client
        .publish_sensor(prefix, "node1", "temperature", &packet.n1.t.to_string(), true)
        .await?;
    mqtt_client
        .publish_sensor(prefix, "node1", "humidity", &packet.n1.h.to_string(), true)
        .await?;
    mqtt_client
        .publish_sensor(prefix, "node1", "gas_resistance", &packet.n1.g.to_string(), true)
        .await?;

    // Node 2 sensors (gateway local sensor - SHT3x)
    if let Some(temp) = packet.n2.t {
        mqtt_client
            .publish_sensor(prefix, "node2", "temperature", &temp.to_string(), true)
            .await?;
    }
    if let Some(humidity) = packet.n2.h {
        mqtt_client
            .publish_sensor(prefix, "node2", "humidity", &humidity.to_string(), true)
            .await?;
    }

    // Signal quality metrics
    mqtt_client
        .publish_sensor(prefix, "signal", "rssi", &packet.sig.rssi.to_string(), false)
        .await?;
    mqtt_client
        .publish_sensor(prefix, "signal", "snr", &packet.sig.snr.to_string(), false)
        .await?;

    // Statistics
    mqtt_client
        .publish_sensor(prefix, "stats", "packets_received", &packet.sts.rx.to_string(), false)
        .await?;
    mqtt_client
        .publish_sensor(prefix, "stats", "crc_errors", &packet.sts.err.to_string(), false)
        .await?;

    info!("Published telemetry to MQTT topics");
    Ok(())
}

/// Write telemetry packet to InfluxDB
async fn write_telemetry_to_influxdb(
    influxdb_client: &influxdb::InfluxDbClient,
    packet: &TelemetryPacket,
) -> anyhow::Result<()> {
    // Node 1 sensors (remote sensor via LoRa)
    influxdb_client
        .write_sensor("temperature", packet.n1.t as f64, "node1", Some("celsius"))
        .await?;
    influxdb_client
        .write_sensor("humidity", packet.n1.h as f64, "node1", Some("percent"))
        .await?;
    influxdb_client
        .write_sensor("gas_resistance", packet.n1.g as f64, "node1", Some("ohms"))
        .await?;

    // Node 2 sensors (gateway local sensor - SHT3x)
    if let Some(temp) = packet.n2.t {
        influxdb_client
            .write_sensor("temperature", temp as f64, "node2", Some("celsius"))
            .await?;
    }
    if let Some(humidity) = packet.n2.h {
        influxdb_client
            .write_sensor("humidity", humidity as f64, "node2", Some("percent"))
            .await?;
    }

    // Signal quality metrics
    influxdb_client
        .write_sensor("rssi", packet.sig.rssi as f64, "signal", Some("dbm"))
        .await?;
    influxdb_client
        .write_sensor("snr", packet.sig.snr as f64, "signal", Some("db"))
        .await?;

    // Statistics
    influxdb_client
        .write_sensor("packets_received", packet.sts.rx as f64, "stats", None)
        .await?;
    influxdb_client
        .write_sensor("crc_errors", packet.sts.err as f64, "stats", None)
        .await?;

    info!("Wrote telemetry to InfluxDB");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(true)
        .init();

    info!("Week 7 MQTT + InfluxDB Gateway Service starting");

    // Load configuration
    let config = Config::load("config.toml").context("Failed to load config.toml")?;
    info!("Configuration loaded successfully");

    // Create MQTT client
    let mqtt_client = mqtt::MqttClient::new(
        &config.mqtt.broker_url,
        &config.mqtt.client_id,
        config.mqtt.qos,
    )
    .await
    .context("Failed to create MQTT client")?;

    // Create InfluxDB client
    let influxdb_client = influxdb::InfluxDbClient::new(
        &config.influxdb.url,
        &config.influxdb.org,
        &config.influxdb.bucket,
        &config.influxdb.token,
    )
    .context("Failed to create InfluxDB client")?;

    // Test InfluxDB connection
    influxdb_client
        .health_check()
        .await
        .context("InfluxDB health check failed")?;
    info!("InfluxDB connection verified");

    // Publish test message (Phase 2.2)
    mqtt_client
        .publish_test_message(&config.mqtt.topic_prefix)
        .await
        .context("Failed to publish test message")?;
    info!("Test message published successfully");

    info!(
        probe = config.gateway.probe_id,
        chip = config.gateway.chip,
        firmware = config.gateway.firmware_path,
        "Spawning probe-rs subprocess"
    );

    // Spawn probe-rs as subprocess
    let mut child = Command::new("probe-rs")
        .args(&[
            "run",
            "--probe",
            &config.gateway.probe_id,
            "--chip",
            &config.gateway.chip,
            &config.gateway.firmware_path,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit()) // Pass through stderr for errors
        .spawn()
        .context("Failed to spawn probe-rs process")?;

    let stdout = child
        .stdout
        .take()
        .context("Failed to capture probe-rs stdout")?;

    // Create channel for telemetry packets
    let (tx, rx) = mpsc::channel::<TelemetryPacket>(config.gateway.channel_capacity);

    // Spawn parser task
    let parser_handle = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        if let Err(e) = parse_probe_rs_output(reader, tx).await {
            error!(error = %e, "Parser task failed");
        }
    });

    // Spawn processor task
    let topic_prefix = config.mqtt.topic_prefix.clone();
    let processor_handle = tokio::spawn(process_telemetry(rx, mqtt_client, influxdb_client, topic_prefix));

    // Wait for Ctrl+C
    info!("Service running. Press Ctrl+C to stop.");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down gracefully");
        }
        _ = parser_handle => {
            warn!("Parser task ended unexpectedly");
        }
    }

    // Kill probe-rs subprocess
    info!("Killing probe-rs subprocess");
    child.kill().await.ok();

    // Wait for processor to finish
    processor_handle.await.ok();

    info!("Week 6 Async Gateway Service stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_log_line() {
        let line = r#"[INFO] JSON sent via VCP: {"ts":12000,"id":"N2"}\n"#;
        let result = extract_json_from_log_line(line);
        assert_eq!(result, Some(r#"{"ts":12000,"id":"N2"}"#.to_string()));
    }

    #[test]
    fn test_extract_json_no_match() {
        let line = "[INFO] Some other log message";
        let result = extract_json_from_log_line(line);
        assert_eq!(result, None);
    }
}
