# Week 7 Learning Notes - MQTT + InfluxDB Integration

**Date**: 2025-12-28
**Focus**: MQTT publishing, Docker infrastructure, topic design
**Status**: ✅ Phase 3 Complete - MQTT telemetry publishing working

---

## Overview

Week 7 extends the Week 6 async gateway to publish telemetry to external systems (MQTT broker and InfluxDB). Baby steps approach: Docker → MQTT client → Publishing → InfluxDB.

---

## Key Decisions

### Docker vs Native Installation

**Chose Docker** for both Mosquitto and InfluxDB:
- ✅ Isolated environment
- ✅ Easy chaos testing (stop/start containers)
- ✅ Version-controlled configuration
- ✅ Matches production patterns

### MQTT Client: rumqttc

**Why rumqttc**:
- Fully async with Tokio
- Clean API
- Active maintenance

**Critical pattern**: Event loop MUST stay alive

```rust
pub struct MqttClient {
    client: AsyncClient,
    _event_loop_handle: JoinHandle<()>,  // Don't drop!
}
```

If the handle is dropped, the MQTT client stops working.

### Topic Hierarchy

**One topic per metric** (not single topic with JSON):

```
iiot/node1/temperature      -> 27.6
iiot/node1/humidity         -> 54.1
iiot/signal/rssi            -> -39
```

**Benefits**:
- Native MQTT filtering (`iiot/#`, `iiot/+/temperature`)
- Simple string values
- Easy InfluxDB integration

### QoS and Retain Strategy

**QoS 1** (At Least Once): Good balance for sensor data

**Retain flags**:
- Sensors: `retain=true` → New subscribers see last value
- Stats: `retain=false` → Counters are time-specific

---

## Technical Insights

### 1. Docker Group Membership

**Problem**: `docker compose` fails with "permission denied"

**Solution**: Group membership doesn't apply to current shell
```bash
# Option 1: New shell
newgrp docker

# Option 2: Log out/in
```

### 2. MQTT Event Loop Lifetime

**Wrong**:
```rust
let (client, event_loop) = AsyncClient::new(...);
// event_loop dropped → client broken!
```

**Correct**:
```rust
let handle = tokio::spawn(async move {
    loop { event_loop.poll().await.ok(); }
});
// Store handle in struct
```

### 3. Testing Without Installing mosquitto-clients

Use Docker on same network:
```bash
docker run --rm -it --network wk7-mqtt-influx_iiot-network \
    eclipse-mosquitto:2 mosquitto_sub -h wk7-mosquitto -t "iiot/#" -v
```

### 4. InfluxDB Auto-Initialization

Docker environment variables auto-setup:
```yaml
DOCKER_INFLUXDB_INIT_MODE=setup
DOCKER_INFLUXDB_INIT_ORG=my-org
DOCKER_INFLUXDB_INIT_BUCKET=telemetry
DOCKER_INFLUXDB_INIT_ADMIN_TOKEN=my-super-secret-auth-token
```

No manual UI setup needed!

---

## Implementation Patterns

### Publishing Helper Function

```rust
async fn publish_telemetry_to_mqtt(
    mqtt_client: &MqttClient,
    prefix: &str,
    packet: &TelemetryPacket,
) -> Result<()> {
    mqtt_client.publish_sensor(
        prefix, "node1", "temperature",
        &packet.n1.t.to_string(), true
    ).await?;
    // ... more sensors
    Ok(())
}
```

### Error Handling: Non-Fatal

```rust
if let Err(e) = publish_telemetry_to_mqtt(...).await {
    error!("MQTT publish failed: {}", e);
    // Continue processing - don't crash!
}
```

**Why**: MQTT is "nice to have", not critical path

---

## Performance

**Added latency**: ~30-35ms per telemetry packet
- MQTT publish: <5ms per message
- 9 sensor values: ~20-30ms total

**Memory**: +3 MB (MQTT client + event loop)

**Network**: ~250 bytes/packet = 25 bytes/sec @ 1 packet/10s

---

## Baby Steps That Worked

1. ✅ Set up Docker services
2. ✅ Verify running (`docker compose ps`)
3. ✅ Create MQTT client module
4. ✅ Publish test message
5. ✅ Integrate with telemetry processor
6. ✅ Test with mosquitto_sub

**Time**: ~2 hours total

---

## Lessons Learned

### 1. Infrastructure First

Set up Docker before writing code → Can test immediately, no "is broker running?" issues

### 2. Topic Design Matters

Rejected single topic with JSON payload:
- ✗ Subscribers must parse JSON
- ✗ No native filtering

Chose one topic per metric:
- ✅ MQTT wildcards work
- ✅ Simple values

### 3. Error Resilience

MQTT failures shouldn't crash gateway:
- Log the error
- Continue processing
- Phase 4 will add retry

---

## Next Phases

**Phase 4**: MQTT Resilience
- Offline buffering queue (max 1000)
- Exponential backoff reconnection

**Phase 5-6**: InfluxDB
- Line protocol conversion
- Batched writes

**Phase 7-8**: Testing
- Hardware end-to-end
- Chaos testing

---

## References

- [rumqttc](https://docs.rs/rumqttc/)
- [MQTT 3.1.1 Spec](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/)
- [Mosquitto](https://mosquitto.org/)
- [InfluxDB 2.x](https://docs.influxdata.com/influxdb/v2/)

---

*Last Updated*: 2025-12-28
*Status*: Phase 3 complete, 30% done
