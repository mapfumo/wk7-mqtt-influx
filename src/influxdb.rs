//! InfluxDB client for time-series data storage
//!
//! This module provides an async InfluxDB 2.x client that:
//! - Connects to InfluxDB server
//! - Writes telemetry data in line protocol format
//! - Handles authentication with API tokens
//! - Supports batched writes for efficiency

use anyhow::{Context, Result};
use influxdb2::Client;
use influxdb2::models::DataPoint;
use tracing::info;

/// InfluxDB client for writing telemetry
pub struct InfluxDbClient {
    client: Client,
    bucket: String,
    url: String,
}

impl InfluxDbClient {
    /// Create a new InfluxDB client
    ///
    /// # Arguments
    /// * `url` - InfluxDB server URL (e.g., "http://localhost:8086")
    /// * `org` - Organization name
    /// * `bucket` - Bucket name for data storage
    /// * `token` - Authentication token
    pub fn new(url: &str, org: &str, bucket: &str, token: &str) -> Result<Self> {
        info!(url = url, org = org, bucket = bucket, "Creating InfluxDB client");

        let client = Client::new(url, org, token);

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            url: url.to_string(),
        })
    }

    /// Test connection to InfluxDB with health check
    pub async fn health_check(&self) -> Result<()> {
        info!("Testing InfluxDB connection...");

        // The influxdb2 crate doesn't expose /health or ping endpoints
        // We'll test by querying the ready endpoint using reqwest
        let health_url = format!("{}/health", self.url);

        let response = reqwest::get(&health_url).await
            .context("Failed to connect to InfluxDB health endpoint")?;

        let status = response.status();
        if status.is_success() {
            info!(status = %status, "InfluxDB health check passed");
            Ok(())
        } else {
            anyhow::bail!("InfluxDB health check failed with status: {}", status)
        }
    }

    /// Write a single data point to InfluxDB
    ///
    /// # Arguments
    /// * `measurement` - Measurement name (e.g., "temperature", "humidity")
    /// * `field_name` - Field name (e.g., "value")
    /// * `field_value` - Field value
    /// * `tags` - Optional tags as key-value pairs
    pub async fn write_point(
        &self,
        measurement: &str,
        field_name: &str,
        field_value: f64,
        tags: Vec<(&str, &str)>,
    ) -> Result<()> {
        // Build data point
        let mut point = DataPoint::builder(measurement)
            .field(field_name, field_value);

        // Add tags
        for (key, value) in tags {
            point = point.tag(key, value);
        }

        let point = point.build()?;

        // Write to InfluxDB
        self.client
            .write(&self.bucket, futures::stream::iter(vec![point]))
            .await
            .context("Failed to write data point to InfluxDB")?;

        info!(
            measurement = measurement,
            field = field_name,
            value = field_value,
            "Wrote data point to InfluxDB"
        );

        Ok(())
    }

    /// Write telemetry sensor value to InfluxDB
    ///
    /// This is a convenience wrapper around write_point for sensor data
    ///
    /// # Arguments
    /// * `sensor_type` - Type of sensor (e.g., "temperature", "humidity")
    /// * `value` - Sensor reading
    /// * `node_id` - Node identifier (e.g., "node1", "node2")
    /// * `unit` - Optional unit tag (e.g., "celsius", "percent")
    pub async fn write_sensor(
        &self,
        sensor_type: &str,
        value: f64,
        node_id: &str,
        unit: Option<&str>,
    ) -> Result<()> {
        let mut tags = vec![("node", node_id)];

        if let Some(u) = unit {
            tags.push(("unit", u));
        }

        self.write_point(sensor_type, "value", value, tags).await
    }
}

#[cfg(test)]
mod tests {
    use super::*

;

    #[test]
    fn test_influxdb_client_creation() {
        let client = InfluxDbClient::new(
            "http://localhost:8086",
            "my-org",
            "telemetry",
            "test-token"
        );
        assert!(client.is_ok());
    }
}
