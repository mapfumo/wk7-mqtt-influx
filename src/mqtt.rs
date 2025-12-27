//! MQTT Client for publishing telemetry data
//!
//! This module provides an async MQTT client that:
//! - Connects to a Mosquitto broker
//! - Publishes sensor telemetry to topic hierarchy
//! - Handles connection errors gracefully
//! - Supports QoS levels and retain flags

use anyhow::{Context, Result};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// MQTT Client for publishing telemetry
pub struct MqttClient {
    client: AsyncClient,
    _event_loop_handle: JoinHandle<()>,
}

impl MqttClient {
    /// Create a new MQTT client and connect to broker
    ///
    /// # Arguments
    /// * `broker_url` - URL like "mqtt://localhost:1883"
    /// * `client_id` - Unique client identifier
    /// * `qos` - Quality of Service level (0, 1, or 2)
    pub async fn new(broker_url: &str, client_id: &str, _qos: u8) -> Result<Self> {
        info!(broker = broker_url, client_id = client_id, "Connecting to MQTT broker");

        // Parse broker URL
        let (host, port) = parse_broker_url(broker_url)?;

        // Configure MQTT options
        let mut mqttoptions = MqttOptions::new(client_id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));

        // Create client
        let (client, mut event_loop) = AsyncClient::new(mqttoptions, 10);

        // Spawn event loop handler task
        let event_loop_handle = tokio::spawn(async move {
            info!("MQTT event loop started");
            loop {
                match event_loop.poll().await {
                    Ok(notification) => {
                        debug!("MQTT notification: {:?}", notification);
                    }
                    Err(e) => {
                        error!("MQTT connection error: {}", e);
                        // Add exponential backoff here in Phase 4
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });

        info!("MQTT client connected successfully");

        Ok(Self {
            client,
            _event_loop_handle: event_loop_handle,
        })
    }

    /// Publish a message to a topic
    ///
    /// # Arguments
    /// * `topic` - MQTT topic (e.g., "iiot/node1/temperature")
    /// * `payload` - Message payload as string
    /// * `qos` - Quality of Service level
    /// * `retain` - Whether to retain this message on the broker
    pub async fn publish(
        &self,
        topic: &str,
        payload: &str,
        qos: QoS,
        retain: bool,
    ) -> Result<()> {
        self.client
            .publish(topic, qos, retain, payload.as_bytes())
            .await
            .with_context(|| format!("Failed to publish to topic: {}", topic))?;

        debug!(topic = topic, payload_len = payload.len(), "Published to MQTT");
        Ok(())
    }

    /// Publish a test message (for Phase 2.2 testing)
    pub async fn publish_test_message(&self, topic_prefix: &str) -> Result<()> {
        let topic = format!("{}/test", topic_prefix);
        let payload = "Hello from Week 7 Gateway Service!";

        info!(topic = %topic, "Publishing test message");

        self.publish(&topic, payload, QoS::AtLeastOnce, false)
            .await?;

        Ok(())
    }

    /// Publish a sensor value to its topic
    ///
    /// # Arguments
    /// * `prefix` - Topic prefix (e.g., "iiot")
    /// * `node` - Node identifier (e.g., "node1", "node2", "signal", "stats")
    /// * `metric` - Metric name (e.g., "temperature", "humidity")
    /// * `value` - Sensor value as string
    /// * `retain` - Whether to retain the value on the broker
    pub async fn publish_sensor(
        &self,
        prefix: &str,
        node: &str,
        metric: &str,
        value: &str,
        retain: bool,
    ) -> Result<()> {
        let topic = Self::build_topic(prefix, node, metric);
        self.publish(&topic, value, QoS::AtLeastOnce, retain).await
    }

    /// Build topic name for a sensor reading
    ///
    /// Topic hierarchy:
    /// - iiot/node1/temperature
    /// - iiot/node1/humidity
    /// - iiot/node1/gas_resistance
    /// - iiot/node2/temperature
    /// - iiot/node2/pressure
    /// - iiot/signal/rssi
    /// - iiot/signal/snr
    /// - iiot/stats/packets_received
    /// - iiot/stats/crc_errors
    pub fn build_topic(prefix: &str, node: &str, metric: &str) -> String {
        format!("{}/{}/{}", prefix, node, metric)
    }
}

/// Parse MQTT broker URL into host and port
///
/// Supports:
/// - mqtt://localhost:1883
/// - mqtt://192.168.1.100:1883
/// - mqtts://broker.example.com:8883 (TLS, for future)
fn parse_broker_url(url: &str) -> Result<(String, u16)> {
    // Remove protocol prefix
    let url_without_protocol = url
        .strip_prefix("mqtt://")
        .or_else(|| url.strip_prefix("mqtts://"))
        .context("Invalid MQTT URL: must start with mqtt:// or mqtts://")?;

    // Split host and port
    if let Some((host, port_str)) = url_without_protocol.split_once(':') {
        let port = port_str
            .parse::<u16>()
            .context("Invalid port number in MQTT URL")?;
        Ok((host.to_string(), port))
    } else {
        // Default port if not specified
        Ok((url_without_protocol.to_string(), 1883))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_broker_url() {
        let (host, port) = parse_broker_url("mqtt://localhost:1883").unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 1883);

        let (host, port) = parse_broker_url("mqtt://192.168.1.100:8883").unwrap();
        assert_eq!(host, "192.168.1.100");
        assert_eq!(port, 8883);

        // Default port
        let (host, port) = parse_broker_url("mqtt://broker.local").unwrap();
        assert_eq!(host, "broker.local");
        assert_eq!(port, 1883);

        // Invalid URL
        assert!(parse_broker_url("http://localhost:1883").is_err());
    }

    #[test]
    fn test_build_topic() {
        assert_eq!(
            MqttClient::build_topic("iiot", "node1", "temperature"),
            "iiot/node1/temperature"
        );
        assert_eq!(
            MqttClient::build_topic("iiot", "signal", "rssi"),
            "iiot/signal/rssi"
        );
    }
}
