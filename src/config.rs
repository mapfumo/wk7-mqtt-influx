//! Configuration management for Week 7 Gateway
//!
//! Loads configuration from config.toml with environment variable overrides

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

/// Complete gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub influxdb: InfluxDbConfig,
    pub gateway: GatewayConfig,
}

/// MQTT broker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker_url: String,
    pub client_id: String,
    pub topic_prefix: String,
    pub qos: u8,
}

/// InfluxDB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluxDbConfig {
    pub url: String,
    pub org: String,
    pub bucket: String,
    pub token: String,
}

/// Gateway hardware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub probe_id: String,
    pub chip: String,
    pub firmware_path: String,
    pub channel_capacity: usize,
}

impl Config {
    /// Load configuration from file
    ///
    /// Environment variables override config file values:
    /// - MQTT_PASSWORD: Override MQTT password
    /// - INFLUXDB_TOKEN: Override InfluxDB token
    pub fn load(path: &str) -> Result<Self> {
        // Read config file
        let config_str = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        // Parse TOML
        let mut config: Config = toml::from_str(&config_str)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        // Override with environment variables
        if let Ok(token) = std::env::var("INFLUXDB_TOKEN") {
            tracing::info!("Using INFLUXDB_TOKEN from environment");
            config.influxdb.token = token;
        }

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate MQTT QoS
        if self.mqtt.qos > 2 {
            anyhow::bail!("Invalid MQTT QoS level: {} (must be 0, 1, or 2)", self.mqtt.qos);
        }

        // Validate URLs
        if !self.mqtt.broker_url.starts_with("mqtt://") && !self.mqtt.broker_url.starts_with("mqtts://") {
            anyhow::bail!("Invalid MQTT broker URL: {} (must start with mqtt:// or mqtts://)", self.mqtt.broker_url);
        }

        if !self.influxdb.url.starts_with("http://") && !self.influxdb.url.starts_with("https://") {
            anyhow::bail!("Invalid InfluxDB URL: {} (must start with http:// or https://)", self.influxdb.url);
        }

        // Validate channel capacity
        if self.gateway.channel_capacity == 0 {
            anyhow::bail!("Gateway channel_capacity must be greater than 0");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = Config {
            mqtt: MqttConfig {
                broker_url: "mqtt://localhost:1883".to_string(),
                client_id: "test".to_string(),
                topic_prefix: "iiot".to_string(),
                qos: 1,
            },
            influxdb: InfluxDbConfig {
                url: "http://localhost:8086".to_string(),
                org: "test-org".to_string(),
                bucket: "test-bucket".to_string(),
                token: "test-token".to_string(),
            },
            gateway: GatewayConfig {
                probe_id: "test-probe".to_string(),
                chip: "STM32F446RETx".to_string(),
                firmware_path: "test.bin".to_string(),
                channel_capacity: 100,
            },
        };

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Invalid QoS should fail
        config.mqtt.qos = 3;
        assert!(config.validate().is_err());
        config.mqtt.qos = 1;

        // Invalid MQTT URL should fail
        config.mqtt.broker_url = "invalid://localhost".to_string();
        assert!(config.validate().is_err());
        config.mqtt.broker_url = "mqtt://localhost:1883".to_string();

        // Invalid InfluxDB URL should fail
        config.influxdb.url = "invalid://localhost".to_string();
        assert!(config.validate().is_err());
        config.influxdb.url = "http://localhost:8086".to_string();

        // Zero channel capacity should fail
        config.gateway.channel_capacity = 0;
        assert!(config.validate().is_err());
    }
}
