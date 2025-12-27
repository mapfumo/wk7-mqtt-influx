# Week 7: MQTT + InfluxDB Integration - TODO

## Phase 1: Project Setup ✅ COMPLETED

- [x] Create Week 7 directory structure
- [x] Initialize Cargo project
- [x] Add dependencies (rumqttc, influxdb2, toml)
- [x] Create config.toml.example
- [x] Create Config struct in src/config.rs
- [x] Implement config validation
- [x] Add environment variable support (INFLUXDB_TOKEN)
- [x] Update main.rs to load config
- [x] Test: cargo build succeeds
- [x] Test: config loads from file
- [x] Test: all unit tests pass (3/3)

**Status**: ✅ Complete - Configuration system working, all tests passing

---

## Phase 2: MQTT Client - Basic Connection

### Step 2.1: MQTT Client Setup

- [x] Install Mosquitto broker (using Docker)
  - [x] Created docker-compose.yml with Mosquitto and InfluxDB
  - [x] Created mosquitto/config/mosquitto.conf
  - [x] Allow anonymous connections for development
  - [x] Start Docker services: `docker compose up -d`
  - [x] Verify services: `docker compose ps`
  - [x] Test broker connectivity: `docker compose logs mosquitto`
- [x] Create src/mqtt.rs module
- [x] Implement MqttClient struct
- [x] Add connect() method with connection logging
- [x] Spawn async connection handler task
- [x] Handle connection errors gracefully
- [x] Test: Connect to local broker without errors
- [x] Test: Verified with docker mosquitto_pub/sub

**Test Criteria**:
- Gateway connects to broker without errors
- Connection visible in Mosquitto logs
- Graceful error if broker unavailable

### Step 2.2: Publish Single Test Message

- [x] Add publish() method to MqttClient
- [x] Add publish_test_message() helper
- [x] Integrate into main.rs to publish on startup
- [x] Create test example: examples/test_mqtt.rs
- [x] Create test helper script: test-mqtt-sub.sh
- [x] Create MQTT_TEST_GUIDE.md documentation
- [x] Test with docker mosquitto_sub
- [x] Verify message received successfully

**Test Criteria**:
- ✅ mosquitto_sub receives test message
- ✅ Logs show successful publish
- ✅ QoS respected

**Status**: ✅ Complete - MQTT client working, test messages publishing successfully

---

## Phase 3: MQTT Publishing - Telemetry Data

### Step 3.1: Topic Hierarchy Design

- [x] Document topic structure in mqtt.rs
- [x] Implement publish_sensor() helper function
- [x] Implement build_topic() helper function
- [x] Test: Topic building verified in unit tests

**Topic Structure**:
```
iiot/node1/temperature      -> 27.6
iiot/node1/humidity         -> 54.1
iiot/node1/gas_resistance   -> 88797
iiot/node2/temperature      -> 25.3
iiot/node2/pressure         -> 1013.2
iiot/signal/rssi            -> -39
iiot/signal/snr             -> 13
iiot/stats/packets_received -> 42
iiot/stats/crc_errors       -> 1
```

### Step 3.2: Publish Telemetry to MQTT

- [x] Update telemetry processor to accept MqttClient
- [x] Publish each sensor reading to its topic
- [x] Add error handling for publish failures
- [x] Implement publish_telemetry_to_mqtt() function
- [ ] Test: Run full system (Node 1 + Gateway)
- [ ] Test: mosquitto_sub -t "iiot/#" -v shows all readings

**Test Criteria**:
- All sensor values published to correct topics
- Data matches structured logs
- No panics if broker disconnects

### Step 3.3: QoS and Retain Settings

- [x] Implement QoS 1 (at least once delivery)
- [x] Add retain flag for sensor values (true for sensors, false for stats)
- [ ] Test: New subscriber gets latest retained values

**Test Criteria**:
- New subscriber immediately sees last values
- Retained messages survive broker restart

**Status**: ✅ Code Complete - Ready for hardware testing with Node 1 + Node 2

---

## Phase 4: MQTT Resilience

### Step 4.1: Offline Buffering
- [ ] Implement in-memory queue for failed publishes
- [ ] Add retry logic with bounded queue (max 1000 messages)
- [ ] Queue drops oldest when full
- [ ] Test: Stop broker, verify messages queued
- [ ] Test: Restart broker, verify queue drains

**Test Criteria**:
- Messages buffered when broker offline
- Queue drains when broker returns
- No memory leak with long-term offline

### Step 4.2: Reconnection Logic
- [ ] Implement exponential backoff (1s, 2s, 4s, 8s, max 60s)
- [ ] Add connection state tracking
- [ ] Log reconnection attempts with backoff time
- [ ] Test: Kill broker, restart, verify automatic reconnect

**Test Criteria**:
- Automatic reconnection without manual intervention
- Backoff prevents connection spam
- Clear reconnection timeline in logs

---

## Phase 5: InfluxDB Client - Basic Connection

### Step 5.1: InfluxDB Setup

- [x] Install InfluxDB 2.x (using Docker - included in docker-compose.yml)
  - [x] Auto-configured with initialization settings:
    - Organization: "my-org"
    - Bucket: "telemetry"
    - Admin user: "admin" / "admin123456"
    - API token: "my-super-secret-auth-token"
- [ ] Start Docker services (same as Mosquitto)
- [ ] Access UI: http://localhost:8086
- [ ] Verify setup and login with admin credentials
- [ ] Update config.toml with token if needed

### Step 5.2: InfluxDB Client Setup
- [ ] Create src/influxdb.rs module
- [ ] Implement InfluxDbClient struct
- [ ] Add connect() method
- [ ] Test health check endpoint
- [ ] Handle auth errors gracefully

**Test Criteria**:
- Health check succeeds
- Auth token validated
- Clear error if org/bucket doesn't exist

### Step 5.3: Write Single Test Point
- [ ] Implement write_point() method
- [ ] Write one hardcoded test point
- [ ] Query from InfluxDB UI to verify
- [ ] Test: Data visible in Data Explorer

**Test Criteria**:
- Point visible in InfluxDB Data Explorer
- Timestamp correct
- Tags and fields as expected

---

## Phase 6: InfluxDB Writer - Telemetry Data

### Step 6.1: Line Protocol Conversion
- [ ] Create telemetry_to_line_protocol() function
- [ ] Design tags: node_id, sensor_type
- [ ] Design fields: all numeric values as f64
- [ ] Handle timestamp conversion (ms to RFC3339/nanoseconds)
- [ ] Test: Verify line protocol format

**Line Protocol Example**:
```
temperature,node_id=node1,sensor_type=environmental value=27.6 1703174400000000000
humidity,node_id=node1,sensor_type=environmental value=54.1 1703174400000000000
gas_resistance,node_id=node1,sensor_type=environmental value=88797 1703174400000000000
pressure,node_id=node2,sensor_type=barometric value=1013.2 1703174400000000000
```

### Step 6.2: Write Telemetry Points
- [ ] Update telemetry processor to write to InfluxDB
- [ ] Write all sensor readings as separate points
- [ ] Add error handling
- [ ] Test: Full system run
- [ ] Test: Query from InfluxDB UI

**Test Criteria**:
- All sensor readings in InfluxDB
- Timestamps align with logs
- Can query by node_id tag

### Step 6.3: Batched Writes
- [ ] Implement batching (buffer 10 points before write)
- [ ] Add flush on timeout (5 seconds max delay)
- [ ] Test: Verify batching reduces write calls
- [ ] Test: Timeout flush works

**Test Criteria**:
- Logs show batched writes
- Low-frequency data still written (timeout flush)
- No data loss

---

## Phase 7: Error Handling & Monitoring

### Step 7.1: Structured Error Logging
- [ ] Add error counters for MQTT publish failures
- [ ] Add error counters for InfluxDB write failures
- [ ] Log failed operations with full context
- [ ] Test: Break connections, verify errors logged clearly

### Step 7.2: Metrics Exposure (Prep for Week 8)
- [ ] Add counter: mqtt_publishes_total{status="success|failure"}
- [ ] Add counter: influxdb_writes_total{status="success|failure"}
- [ ] Add gauge: telemetry_packets_processed_total
- [ ] Log metrics to stdout (Week 8 will expose via /metrics)

**Test Criteria**:
- Metrics appear in structured logs
- Counters increment correctly
- Failures properly attributed

---

## Phase 8: Integration Testing

### Step 8.1: End-to-End Test
- [ ] Run full system (Node 1 + Gateway)
- [ ] Verify data in MQTT (mosquitto_sub)
- [ ] Verify data in InfluxDB UI
- [ ] Verify logs match both systems
- [ ] Test Ctrl+C graceful shutdown
- [ ] Test: 10 minutes continuous operation

**Test Checklist**:
- [ ] Node 1 transmitting
- [ ] Gateway receiving and parsing
- [ ] MQTT topics updating in real-time
- [ ] InfluxDB receiving writes
- [ ] No memory leaks (check htop)
- [ ] No errors in logs
- [ ] Graceful shutdown works

### Step 8.2: Chaos Testing
- [ ] Test: Stop/start MQTT broker during operation
- [ ] Test: Stop/start InfluxDB during operation
- [ ] Test: Disconnect Node 1 (sensor offline)
- [ ] Verify: System recovers automatically

**Test Criteria**:
- Reconnection works automatically
- Buffered messages delivered when services recover
- No crashes or data corruption

---

## Phase 9: Documentation

### Step 9.1: Create Documentation Files
- [ ] README.md (architecture, quick start, configuration)
- [ ] QUICKSTART.md (fastest way to run)
- [ ] NOTES.md (learnings about MQTT and InfluxDB)
- [ ] TROUBLESHOOTING.md (common issues and solutions)
- [ ] Update this TODO.md with completion status

### Step 9.2: Update Week 6 References
- [ ] Update Week 6 README to link to Week 7
- [ ] Update main 4-month-plan README with Week 7 status

---

## Phase 10: Optional Enhancements

### 10.1: Environment Variables for Secrets
- [ ] Support MQTT_PASSWORD env var
- [ ] Support INFLUXDB_TOKEN env var (already done)
- [ ] Document in README

### 10.2: Configuration Validation
- [ ] Validate config on startup
- [ ] Fail fast with clear errors
- [ ] Suggest fixes for common issues

### 10.3: Last Will and Testament (LWT)
- [ ] Configure MQTT LWT: iiot/gateway/status = "offline"
- [ ] Publish "online" on connect
- [ ] Test: Kill gateway, verify "offline" published

---

## Current Status

**Phase 1**: ✅ COMPLETED (Project setup, configuration working)
**Phase 2**: ✅ COMPLETED (MQTT client and test messages working)
**Phase 3**: ✅ COMPLETED (Telemetry MQTT publishing implemented - ready for hardware test)
**Phase 4**: 🔲 NOT STARTED (MQTT resilience - offline buffering, reconnection)
**Phase 5**: 🔄 PARTIALLY DONE (InfluxDB Docker setup complete, ready for Phase 5)

**Overall Progress**: 3/10 phases complete (~30%)

**Recent Session Notes**:

- ✅ Docker services running (Mosquitto + InfluxDB)
- ✅ MQTT client module created and tested
- ✅ Telemetry publishing to MQTT topics implemented
- ✅ All sensor values mapped to topic hierarchy
- ✅ QoS 1 and retain flags configured
- ✅ Error handling for MQTT failures
- ✅ All unit tests passing (5/5)
- **Next Step**: Test with live hardware (Node 1 + Node 2) or proceed to Phase 4/5

---

## Notes

- Each phase builds on the previous
- Test thoroughly before moving to next phase
- No TODO comments in code - finish features completely
- Commit after each phase completion
