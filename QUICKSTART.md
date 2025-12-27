# Week 7 Quick Start Guide

**5-minute setup for MQTT + InfluxDB telemetry gateway**

---

## Prerequisites Check

```bash
# Check Docker
docker --version

# Check Rust
rustc --version

# Check hardware connected
lsusb | grep STMicro
```

---

## Step 1: Start Services (30 seconds)

```bash
cd wk7-mqtt-influx

# Start MQTT + InfluxDB
docker compose up -d

# Verify running (should see 2 containers)
docker compose ps
```

Expected output:
```
wk7-influxdb    influxdb:2            Up   0.0.0.0:8086->8086/tcp
wk7-mosquitto   eclipse-mosquitto:2   Up   0.0.0.0:1883->1883/tcp
```

---

## Step 2: Build Firmware (1-2 minutes, one-time)

```bash
# Build Node 2 gateway firmware
./build-firmware.sh
```

Expected output:
```
Finished `release` profile [optimized + debuginfo] target(s) in 21.22s
```

---

## Step 3: Run Gateway (3 seconds)

```bash
# Run gateway service
./run-gateway.sh
```

Expected startup:
```
✓ Configuration loaded successfully
✓ MQTT client connected successfully
✓ InfluxDB health check passed
✓ Test message published successfully
✓ Spawning probe-rs subprocess
✓ Firmware flashing... (3-15 seconds)
✓ Service running. Press Ctrl+C to stop.
```

Wait for telemetry packets (every ~10 seconds):
```
INFO Telemetry packet received node_id=N2 timestamp_ms=8000 temp_c=26.7 humidity_pct=55.4
INFO Published telemetry to MQTT topics
INFO Wrote telemetry to InfluxDB
```

---

## Step 4: Verify Data (Optional)

### Check MQTT (Terminal 2)

```bash
./test-mqtt-sub.sh
```

You should see messages:
```
iiot/node1/temperature 26.7
iiot/node1/humidity 55.4
iiot/signal/rssi -33
...
```

### Check InfluxDB UI

```bash
# Open browser
open http://localhost:8086

# Login
Username: admin
Password: admin123456

# Navigate to: Data Explorer → Bucket: telemetry
# You should see 8 measurements with live data
```

---

## Common Issues

### "Connection refused" on MQTT

```bash
# Check Mosquitto is running
docker compose logs mosquitto

# Restart if needed
docker compose restart mosquitto
```

### "Permission denied" on Docker

```bash
# Apply docker group (if just added)
newgrp docker

# Or restart terminal/log out and back in
```

### No telemetry packets

**Check Node 1 is powered on** - It transmits LoRa packets that Node 2 receives.

**Check logs for errors**:
```bash
# Gateway should show:
# "Telemetry packet received" every ~10 seconds
```

---

## Power Cycling

**Node 1 (remote sensor):**
- ✅ Gateway auto-recovers
- Data resumes when Node 1 boots

**Node 2 (gateway board):**
- ❌ Must restart: `./run-gateway.sh`
- RTT connection breaks when powered off

---

## Stop Everything

```bash
# Stop gateway
Ctrl+C

# Stop Docker services
docker compose down

# Remove all data (optional)
docker compose down -v
```

---

## Scripts Reference

```bash
./build-firmware.sh       # Build Node 2 firmware (one-time)
./run-gateway.sh         # Run complete system
./test-mqtt-sub.sh       # Subscribe to MQTT messages

cargo test                # Run unit tests
cargo build --release     # Build optimized gateway
```

---

## Documentation

- **MQTT_INFLUX_GUIDE.md** - Comprehensive tutorial
- **FINAL_SUMMARY.md** - Complete session summary
- **TROUBLESHOOTING.md** - Detailed troubleshooting
- **README.md** - Full project documentation

---

## Next Steps

1. **View data in InfluxDB UI** - Create queries, dashboards
2. **Monitor MQTT topics** - Connect your own applications
3. **Explore the code** - See how MQTT + InfluxDB integration works
4. **Read MQTT_INFLUX_GUIDE.md** - Understand the architecture

---

**That's it!** You now have a complete IoT telemetry pipeline publishing to both MQTT and InfluxDB.
