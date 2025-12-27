# MQTT Testing Guide

## Prerequisites

- Docker services running: `docker compose up -d`
- Verify: `docker compose ps` shows both mosquitto and influxdb as "Up"

## Manual MQTT Testing

### Test 1: Verify Broker Connectivity

```bash
# Publish a test message using mosquitto_pub
docker run --rm --network wk7-mqtt-influx_iiot-network eclipse-mosquitto:2 \
    mosquitto_pub -h wk7-mosquitto -t "iiot/test" -m "Hello MQTT"
```

### Test 2: Subscribe to Messages

Open a terminal and run:

```bash
# Subscribe to all iiot topics
docker run --rm -it --network wk7-mqtt-influx_iiot-network eclipse-mosquitto:2 \
    mosquitto_sub -h wk7-mosquitto -t "iiot/#" -v
```

Or use the helper script:

```bash
./test-mqtt-sub.sh
```

Leave this running to see published messages.

### Test 3: Test Rust MQTT Client

In another terminal, run the test example:

```bash
cargo run --example test_mqtt
```

Expected output:
```
INFO MQTT Test Program Starting
INFO Configuration loaded successfully
INFO Connecting to MQTT broker broker=mqtt://localhost:1883 client_id=iiot-gateway
INFO MQTT client connected
INFO MQTT event loop started
INFO Publishing test message topic=iiot/test
INFO ✓ Test message published successfully!
INFO Test complete
```

In the subscriber terminal, you should see:
```
iiot/test Hello from Week 7 Gateway Service!
```

### Test 4: Full Gateway Service (requires hardware)

This test requires Node 2 firmware built and hardware connected:

```bash
cargo run
```

The service will:
1. Connect to MQTT broker
2. Publish test message to `iiot/test`
3. Spawn probe-rs for Node 2 firmware
4. Parse telemetry and publish to MQTT topics

## MQTT Topic Hierarchy

The gateway publishes to these topics:

- `iiot/test` - Test messages
- `iiot/node1/temperature` - Node 1 temperature (°C)
- `iiot/node1/humidity` - Node 1 humidity (%)
- `iiot/node1/gas_resistance` - Node 1 gas resistance (ohms)
- `iiot/node2/temperature` - Node 2 temperature (°C)
- `iiot/node2/pressure` - Node 2 pressure (hPa)
- `iiot/signal/rssi` - LoRa RSSI (dBm)
- `iiot/signal/snr` - LoRa SNR (dB)
- `iiot/stats/packets_received` - Packet count
- `iiot/stats/crc_errors` - Error count

## Troubleshooting

### Broker Not Accessible

```bash
# Check if Mosquitto is running
docker compose ps mosquitto

# Check logs
docker compose logs mosquitto

# Restart if needed
docker compose restart mosquitto
```

### No Messages Received

1. Verify network: `docker network ls | grep iiot`
2. Check subscriber is on same network
3. Verify broker URL in config.toml matches container name

### Connection Refused

- Ensure port 1883 is exposed: `docker compose ps` should show `0.0.0.0:1883->1883/tcp`
- Check firewall settings

## Clean Up

```bash
# Stop subscriber
docker stop mqtt-test-sub

# Stop all services
docker compose down

# Remove volumes (resets InfluxDB)
docker compose down -v
```
