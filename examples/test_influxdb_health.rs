//! Test InfluxDB health check
//!
//! Run with: cargo run --example test_influxdb_health

use wk7_mqtt_influx::influxdb::InfluxDbClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    // Create InfluxDB client with config from docker-compose.yml
    let client = InfluxDbClient::new(
        "http://localhost:8086",
        "my-org",
        "telemetry",
        "my-super-secret-auth-token",
    )?;

    // Test health check
    client.health_check().await?;
    println!("✅ InfluxDB health check passed!");

    // Write a test data point
    println!("\nWriting test data point...");
    client.write_point(
        "test_measurement",
        "value",
        42.5,
        vec![("source", "test"), ("node", "test_node")]
    ).await?;
    println!("✅ Test data point written!");

    println!("\nCheck InfluxDB UI at http://localhost:8086");
    println!("  Organization: my-org");
    println!("  Bucket: telemetry");
    println!("  Measurement: test_measurement");

    Ok(())
}
