#!/bin/bash
# Test MQTT connection without hardware
# This will test just the MQTT client initialization and test message

echo "Testing MQTT connection..."
echo ""

# Create a minimal test program
cat > /tmp/test_mqtt.rs << 'EOF'
use wk7_mqtt_influx::mqtt::MqttClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    println!("Creating MQTT client...");
    let client = MqttClient::new("mqtt://localhost:1883", "test-client", 1).await?;

    println!("Publishing test message...");
    client.publish_test_message("iiot").await?;

    println!("✓ Test message published successfully!");

    // Give time for message to be sent
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    Ok(())
}
EOF

echo "Test program created. To test MQTT manually, run:"
echo ""
echo "  Terminal 1: ./test-mqtt-sub.sh"
echo "  Terminal 2: cargo run"
echo ""
echo "Or use Docker to subscribe:"
echo "  docker run --rm -it --network wk7-mqtt-influx_iiot-network eclipse-mosquitto:2 mosquitto_sub -h wk7-mosquitto -t 'iiot/#' -v"
