# Grafana Setup Guide - Week 8

**Project**: wk7-mqtt-influx (Week 7 + Week 8 Combined)
**Date**: 2025-12-28
**Purpose**: Step-by-step Grafana configuration for IoT telemetry visualization

---

## Overview

This guide covers:
1. Accessing Grafana UI
2. Configuring InfluxDB data source
3. Creating sensor telemetry dashboard
4. Setting up alerts
5. Testing and verification

**Philosophy**: Baby steps - verify each step before proceeding!

---

## Phase 2: Configure InfluxDB Data Source (10 minutes)

### Step 1: Access Grafana UI

**Open in browser:**
```
http://localhost:3000
```

**Expected**: Grafana login page appears

**If fails**:
- Check Grafana is running: `docker compose ps | grep grafana`
- Check logs: `docker compose logs grafana`

---

### Step 2: Login to Grafana

**Default credentials:**
- **Username**: `admin`
- **Password**: `admin`

**On first login**: Grafana may prompt you to change password
- You can skip this for development (click "Skip")
- Or set a new password

**Expected**: Grafana home dashboard appears

---

### Step 3: Navigate to Data Sources

**Navigation path:**
1. Click the **gear icon** (⚙️) in left sidebar → "Configuration"
2. Click **"Data sources"**
3. Click **"Add data source"** button

**Expected**: List of available data source types appears

---

### Step 4: Select InfluxDB

**Steps:**
1. Find **"InfluxDB"** in the list (use search box if needed)
2. Click on **"InfluxDB"**

**Expected**: InfluxDB configuration page appears

---

### Step 5: Configure InfluxDB Connection

**IMPORTANT**: Use exact values below - copy/paste to avoid typos!

#### Query Language
- Select: **Flux** (NOT InfluxQL)

#### HTTP Section

**URL:**
```
http://wk7-influxdb:8086
```

**CRITICAL**: Must use container name `wk7-influxdb`, NOT `localhost`!
**Why**: Grafana container talks to InfluxDB container via Docker network.

**Access:**
- Keep default: **Server (default)**

#### InfluxDB Details

**Organization:**
```
my-org
```

**Token:**
```
my-super-secret-auth-token
```

**Default Bucket:**
```
telemetry
```

**Min time interval:**
- Leave empty (use default)

---

### Step 6: Test Connection

**Steps:**
1. Scroll to bottom of page
2. Click **"Save & test"** button

**Expected Success Message:**
```
✓ datasource is working. 8 measurements found
```

**If you see success**: ✅ Proceed to Step 7

**If test fails**: See troubleshooting section below

---

### Step 7: Verify Measurements

**After successful test:**

1. Stay on data source configuration page
2. Look for message showing measurements found
3. Should see **8 measurements**:
   - temperature
   - humidity
   - gas_resistance
   - pressure
   - rssi
   - snr
   - packets_received
   - crc_errors

**Expected**: All 8 measurements from your telemetry data

---

## Verification - Data Source Is Working

### Quick Test Query

**Steps:**
1. From Grafana home, click **"Explore"** (compass icon in left sidebar)
2. Select **"InfluxDB"** as data source (top dropdown)
3. Click **"Script editor"** button (if in visual mode)
4. Enter this Flux query:

```flux
from(bucket: "telemetry")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> filter(fn: (r) => r.node == "node1")
```

5. Click **"Run query"** button (or Shift+Enter)

**Expected Result:**
- Graph appears showing temperature data from Node 1
- Time series line chart with temperature values (~26-28°C)
- Data points from last hour

**If you see the graph**: ✅ **InfluxDB data source is fully working!**

---

## Troubleshooting

### Error: "Failed to call resource"

**Symptom**: Connection test fails with network error

**Causes & Fixes:**

1. **Wrong URL**
   - ❌ `http://localhost:8086` (won't work from Grafana container)
   - ✅ `http://wk7-influxdb:8086` (use container name)

2. **InfluxDB not running**
   ```bash
   docker compose ps | grep influxdb
   # Should show "Up"
   ```

   If not running:
   ```bash
   docker compose up -d influxdb
   ```

3. **Network issue**
   ```bash
   # Test from Grafana container
   docker exec wk7-grafana wget -O- http://wk7-influxdb:8086/health
   ```

   Should return: `{"name":"influxdb","message":"ready for queries and writes","status":"pass"}`

---

### Error: "Failed to authenticate"

**Symptom**: Connection fails with auth error

**Causes & Fixes:**

1. **Wrong token**
   - Check token in [docker-compose.yml](docker-compose.yml:30)
   - Must match: `my-super-secret-auth-token`
   - Copy/paste to avoid typos

2. **Wrong organization**
   - Must be: `my-org`
   - Case-sensitive!

---

### Error: "Bucket not found"

**Symptom**: Connection works but can't query data

**Causes & Fixes:**

1. **Wrong bucket name**
   - Must be: `telemetry`
   - Check in InfluxDB UI: http://localhost:8086

2. **Bucket is empty**
   ```bash
   # Verify data exists
   docker exec wk7-influxdb influx query \
     'from(bucket:"telemetry") |> range(start: -1h) |> limit(n:1)' \
     --org my-org --token my-super-secret-auth-token
   ```

   If empty: Run gateway to collect data
   ```bash
   ./run-gateway.sh
   ```

---

### No Measurements Found

**Symptom**: Test passes but 0 measurements shown

**Causes:**

1. **No data in bucket yet**
   - Gateway hasn't run, or
   - InfluxDB was recently cleared

**Fix:**
```bash
# Run gateway to generate telemetry data
./run-gateway.sh

# Wait 30 seconds for a few packets
# Then re-test data source
```

---

## Configuration Summary

**Completed Configuration:**

| Setting | Value |
|---------|-------|
| Data Source Type | InfluxDB |
| Query Language | Flux |
| URL | http://wk7-influxdb:8086 |
| Organization | my-org |
| Token | my-super-secret-auth-token |
| Default Bucket | telemetry |
| Status | ✅ Working |

---

## Available Measurements

After configuration, you can query these measurements:

| Measurement | Tags | Field | Unit | Source |
|-------------|------|-------|------|--------|
| temperature | node=node1, unit=celsius | value | °C | BME680 |
| humidity | node=node1, unit=percent | value | % | BME680 |
| gas_resistance | node=node1, unit=ohms | value | Ω | BME680 |
| temperature | node=node2, unit=celsius | value | °C | BMP280 |
| pressure | node=node2, unit=hpa | value | hPa | BMP280 |
| rssi | node=signal, unit=dbm | value | dBm | LoRa |
| snr | node=signal, unit=db | value | dB | LoRa |
| packets_received | node=stats | value | count | Gateway |
| crc_errors | node=stats | value | count | Gateway |

**All measurements** have:
- **Tag**: `node` (for filtering by source)
- **Tag**: `unit` (for measurements with units)
- **Field**: `value` (the actual reading)
- **Timestamp**: Automatically recorded by InfluxDB

---

## Example Queries

### Temperature from Node 1
```flux
from(bucket: "telemetry")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> filter(fn: (r) => r.node == "node1")
```

### Compare Temperature: Node 1 vs Node 2
```flux
from(bucket: "telemetry")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> filter(fn: (r) => r.node == "node1" or r.node == "node2")
```

### Signal Quality (RSSI)
```flux
from(bucket: "telemetry")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "rssi")
```

### All Sensor Data (Last 10 Minutes)
```flux
from(bucket: "telemetry")
  |> range(start: -10m)
```

---

## Next Steps

After InfluxDB data source is configured and tested:

1. ✅ **Phase 2 Complete**: Data source working
2. ➡️ **Phase 3**: Create Dashboard 1 - Sensor Telemetry
   - Temperature comparison panel
   - Humidity panel
   - Pressure panel
   - Signal quality panels
   - Packet statistics

See **Phase 3** in [WEEK_8_PLAN.md](../DOCS/WEEK_8_PLAN.md) for dashboard creation guide.

---

## Quick Commands Reference

```bash
# Check Grafana status
docker compose ps grafana

# View Grafana logs
docker compose logs grafana

# Restart Grafana
docker compose restart grafana

# Access Grafana UI
open http://localhost:3000

# Test InfluxDB from command line
docker exec wk7-influxdb influx query \
  'from(bucket:"telemetry") |> range(start: -1h) |> limit(n:5)' \
  --org my-org --token my-super-secret-auth-token
```

---

## Success Checklist - Phase 2

- [ ] Grafana UI accessible at http://localhost:3000
- [ ] Logged in successfully (admin/admin)
- [ ] InfluxDB data source added
- [ ] Connection test passed
- [ ] 8 measurements found
- [ ] Test query in Explore shows data
- [ ] Temperature graph displays correctly

**When all checked**: ✅ **Phase 2 Complete - Ready for Dashboard Creation!**

---

*Following the baby steps philosophy - each step verified before proceeding!*
