# Grafana Dashboard Guide - IoT Sensor Telemetry

**Project**: wk7-mqtt-influx
**Week 8 - Phase 3**: Create Sensor Telemetry Dashboard
**Date**: 2025-12-28
**Status**: ✅ Data flowing from sensors → MQTT → InfluxDB → Grafana

---

## Overview

This comprehensive guide walks you through creating professional IoT sensor telemetry dashboards with live data from your STM32 nodes.

**What you'll build:**
- Dashboard 1: Environmental Sensors (temperature, humidity, air quality)
- Dashboard 2: Signal Quality & Statistics (RSSI, SNR, packet stats)

**Time estimate**: 30-45 minutes (taking it slow, baby steps!)

**Philosophy**: Build one panel at a time, verify it works, then move to the next.

---

## Quick Access

- **Grafana UI**: http://localhost:3000
- **Default Login**: admin / admin (change on first login)
- **InfluxDB UI**: http://localhost:8086
- **MQTT Broker**: mqtt://localhost:1883

---

## Prerequisites

Before starting:
- ✅ Docker Compose stack running (`docker compose up -d`)
- ✅ Gateway service collecting telemetry
- ✅ InfluxDB receiving data (verify with `docker compose logs wk7-influxdb`)
- ✅ Both Node 1 and Node 2 transmitting/receiving

---

# Part 1: Configure InfluxDB Data Source

## Step 1: Access Data Sources (2 min)

**In Grafana UI:**

1. Open browser: http://localhost:3000
2. Login with: `admin` / `admin`
3. **Change password** when prompted (or skip)
4. Navigate to: **⚙️ Configuration** (left sidebar) → **Data Sources**
5. Click: **Add data source**
6. Select: **InfluxDB**

**Expected**: InfluxDB configuration page appears

---

## Step 2: Configure Connection (3 min)

### Query Language

**IMPORTANT**: Select **Flux** from the dropdown

⚠️ Do NOT use InfluxQL - our data uses Flux queries

### HTTP Settings

**URL**: `http://wk7-influxdb:8086`

⚠️ **CRITICAL**: Use container name `wk7-influxdb`, NOT `localhost`
This is the Docker internal network hostname.

**Other HTTP settings**:
- Auth: Leave all checkboxes unchecked
- Timeout: 60 seconds (default)

### InfluxDB Details

- **Organization**: `my-org`
- **Token**: `my-super-secret-auth-token`
- **Default Bucket**: `telemetry`

**Where these values come from**: [docker-compose.yml](docker-compose.yml#L40-L50)

---

## Step 3: Test Connection (1 min)

1. Scroll to bottom
2. Click **Save & Test**
3. Wait for response

**Expected**: ✅ Green checkmark with **"Data source is working"**

**If it fails**:
- Double-check container name: `wk7-influxdb` (not localhost)
- Verify token matches [docker-compose.yml](docker-compose.yml:43)
- Check containers are running: `docker compose ps`
- View InfluxDB logs: `docker compose logs wk7-influxdb`

---

# Part 2: Dashboard 1 - Environmental Sensors

This dashboard shows:
- Temperature comparison (Node 1 vs Node 2) - both have SHT3x sensors
- Humidity comparison (Node 1 vs Node 2) - both have SHT3x sensors
- Gas resistance (Node 1 only - BME680 air quality sensor)
- Current values (stat panels)

---

## Step 1: Create New Dashboard (2 min)

**In Grafana UI:**

1. Click the **"+"** button in the left sidebar
2. Select **"Create Dashboard"**
3. You'll see a blank dashboard with "Add visualization" button

**Expected**: Empty dashboard canvas

---

## Step 2: Panel 1 - Temperature Comparison (5 min)

**Goal**: Show temperature from both nodes on the same graph

### Create the Panel

1. Click **"Add visualization"**
2. Select your **"InfluxDB"** data source
3. You'll see the query editor at the bottom

### Configure Query A (Node 1 Temperature)

Click **"Code"** button (top-right of query editor) to switch to code mode

**Query A - Node 1 Temperature:**
```flux
from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> filter(fn: (r) => r.node == "node1")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
  |> yield(name: "Node 1")
```

**What this does:**
- `from(bucket: "telemetry")` - Query your data bucket
- `range(start: v.timeRangeStart, stop: v.timeRangeStop)` - Use dashboard time range
- `filter(fn: (r) => r._measurement == "temperature")` - Get temperature data
- `filter(fn: (r) => r.node == "node1")` - Only Node 1
- `aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)` - Downsample for performance
- `yield(name: "Node 1")` - Label the series

### Add Query B (Node 2 Temperature)

1. Click **"+ Query"** button below Query A
2. Switch to **"Code"** mode

**Query B - Node 2 Temperature:**
```flux
from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> filter(fn: (r) => r.node == "node2")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
  |> yield(name: "Node 2")
```

### Configure Panel Settings

**Panel options** (right sidebar):
- **Title**: `Temperature Comparison`
- **Description**: `SHT3x sensors on both nodes`

**Standard options**:
- **Unit**: `Temperature > Celsius (°C)`
- **Decimals**: `1`

**Graph styles**:
- **Line width**: `2`
- **Fill opacity**: `10`

**Legend**:
- **Visibility**: `Show`
- **Placement**: `Bottom`
- **Display mode**: `List`
- **Values**: Check `Last` (shows most recent value)

### Save Panel

1. Click **"Apply"** button (top-right)
2. You should see your temperature graph!

**Expected**: Two lines showing Node 1 (~28.4°C) and Node 2 (~27.9°C) temperatures over time

---

## Step 3: Panel 2 - Humidity Comparison (5 min)

**Goal**: Show humidity from both nodes (both have SHT3x sensors!)

### Create Panel

1. Click **"Add"** button (top toolbar) → **"Visualization"**
2. Select **"InfluxDB"** data source

### Configure Query A (Node 1 Humidity)

Switch to **"Code"** mode

**Query A:**
```flux
from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "humidity")
  |> filter(fn: (r) => r.node == "node1")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
  |> yield(name: "Node 1")
```

### Add Query B (Node 2 Humidity)

1. Click **"+ Query"**
2. Switch to **"Code"** mode

**Query B:**
```flux
from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "humidity")
  |> filter(fn: (r) => r.node == "node2")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
  |> yield(name: "Node 2")
```

### Panel Settings

**Panel options:**
- **Title**: `Humidity Comparison`
- **Description**: `SHT3x relative humidity (both nodes)`

**Standard options:**
- **Unit**: `Humidity > Percent (0-100)`
- **Min**: `0`
- **Max**: `100`
- **Decimals**: `1`

**Thresholds** (optional, for color coding):
- Add threshold: `30` → Color: Yellow (dry)
- Add threshold: `40` → Color: Green (comfortable)
- Add threshold: `70` → Color: Yellow (humid)

**Graph styles:**
- **Line width**: `2`
- **Fill opacity**: `20`
- **Gradient mode**: `Opacity`

### Save Panel

Click **"Apply"**

**Expected**: Two lines showing Node 1 (~51.7%) and Node 2 (~48.7%) humidity

---

## Step 4: Panel 3 - Gas Resistance (4 min)

**Goal**: Show air quality sensor reading from Node 1 (BME680 only)

### Create Panel

1. **"Add"** → **"Visualization"** → **"InfluxDB"**

### Query

```flux
from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "gas_resistance")
  |> filter(fn: (r) => r.node == "node1")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
```

### Panel Settings

**Panel options:**
- **Title**: `Air Quality - Gas Resistance (Node 1)`
- **Description**: `BME680 VOC sensor - higher resistance = better air quality`

**Standard options:**
- **Unit**: `None` (it's in Ohms, displayed as ~80k-100k)
- **Decimals**: `0`

**Graph styles:**
- **Line width**: `2`
- **Fill opacity**: `15`
- **Color**: Green

**Info note**: Higher resistance = cleaner air. Decreases when VOCs present (breathing, cooking, etc.)

### Save Panel

Click **"Apply"**

**Expected**: Green line showing resistance values (~80,000-100,000 Ω)

---

## Step 5: Panel 4 - Current Values (Stat Panels) (8 min)

**Goal**: Create 4 stat panels side-by-side showing current sensor readings

We'll create these as separate panels positioned next to each other.

### Panel 4A: Node 1 Temperature (Current)

1. **"Add"** → **"Visualization"**
2. **Change visualization type**: Click **"Time series"** dropdown → Select **"Stat"**
3. Select **"InfluxDB"** data source

**Query:**
```flux
from(bucket: "telemetry")
  |> range(start: -5m)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> filter(fn: (r) => r.node == "node1")
  |> last()
```

**Panel options:**
- **Title**: `Node 1 Temperature`

**Standard options:**
- **Unit**: `Temperature > Celsius (°C)`
- **Decimals**: `1`

**Stat styles:**
- **Orientation**: `Horizontal`
- **Text mode**: `Value and name`
- **Color mode**: `Background`

**Thresholds** (color-based):
- Base: Green (default)
- Add: `30` → Yellow
- Add: `35` → Red

Click **"Apply"**

### Panel 4B: Node 1 Humidity (Current)

Repeat the same process:

**Query:**
```flux
from(bucket: "telemetry")
  |> range(start: -5m)
  |> filter(fn: (r) => r._measurement == "humidity")
  |> filter(fn: (r) => r.node == "node1")
  |> last()
```

**Panel options:**
- **Title**: `Node 1 Humidity`

**Unit**: `Humidity > Percent (0-100)`

**Thresholds**:
- Base: Red
- Add: `30` → Yellow
- Add: `40` → Green
- Add: `70` → Yellow
- Add: `80` → Red

Click **"Apply"**

### Panel 4C: Node 2 Temperature (Current)

Same as Panel 4A, but change filter to `node == "node2"`

**Title**: `Node 2 Temperature`

### Panel 4D: Node 2 Humidity (Current)

Same as Panel 4B, but change filter to `node == "node2"`

**Title**: `Node 2 Humidity`

---

## Step 6: Arrange Panel Layout (5 min)

**Goal**: Organize panels in a clean grid

Grafana uses a **24-column grid**. Drag panels to arrange them.

**Recommended layout:**

```
┌─────────────────────────────────────────────────────────────────┐
│ Environmental Sensors - Live Data             [Auto-refresh: 10s]│
├────────────────────┬────────────────────┬──────────────┬─────────┤
│ Node 1 Temp        │ Node 1 Humidity    │ Node 2 Temp  │ Node 2  │
│ 28.4°C ↑           │ 51.7% →            │ 27.9°C ↑     │ 48.7% ↓ │
│ (Stat panel)       │ (Stat panel)       │ (Stat panel) │ (Stat)  │
├────────────────────┴────────────────────┴──────────────┴─────────┤
│ Temperature Comparison (Node 1 vs Node 2)                        │
│ ┌───────────────────────────────────────────────────────────┐    │
│ │ 29°C ────────── Node 1                                    │    │
│ │ 28°C ──────────                                           │    │
│ │ 27°C ───────── Node 2                                     │    │
│ │      12:00      13:00      14:00                          │    │
│ └───────────────────────────────────────────────────────────┘    │
├──────────────────────────────────────────────────────────────────┤
│ Humidity Comparison (Node 1 vs Node 2)                           │
│ ┌───────────────────────────────────────────────────────────┐    │
│ │ 52% ────────── Node 1                                     │    │
│ │ 50% ──────────                                            │    │
│ │ 48% ───────── Node 2                                      │    │
│ │      12:00      13:00      14:00                          │    │
│ └───────────────────────────────────────────────────────────┘    │
├──────────────────────────────────────────────────────────────────┤
│ Air Quality - Gas Resistance (Node 1)                            │
│ ┌───────────────────────────────────────────────────────────┐    │
│ │ 100kΩ ──────────                                          │    │
│ │  80kΩ ────────────                                        │    │
│ │      12:00      13:00      14:00                          │    │
│ └───────────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────────┘
```

**Sizing guide:**
- **Stat panels** (Row 1): 6 columns each (24÷4 = 6 columns per panel)
- **Time series graphs** (Rows 2-4): 24 columns (full width)
- **Panel height**: Stat panels ~4-5 units, graphs ~8-10 units

**How to resize/move:**
1. Click panel title → Drag to move
2. Drag bottom-right corner to resize
3. Panels snap to grid automatically

---

## Step 7: Configure Dashboard Settings (3 min)

### Click ⚙️ gear icon (top toolbar) → "Settings"

**General:**
- **Name**: `Environmental Sensors`
- **Description**: `Live telemetry from Node 1 and Node 2 via LoRa - Temperature, Humidity, Air Quality`
- **Tags**: Add `iot`, `sensors`, `lora`, `week8` (press Enter after each)

**Time options:**
- **Timezone**: `Browser Time`
- **Auto refresh**: `10s` (or `5s` for more real-time feel)
- **Refresh live dashboards**: ✓ Enabled
- **Hide time picker**: ❌ (leave unchecked)

### Save Dashboard

Click **"Save dashboard"** (💾 icon top-right)
- **Save message**: "Initial environmental sensors dashboard"
- Click **"Save"**

---

## Step 8: Set Time Range (1 min)

**Top-right time picker:**

For live monitoring, recommended:
- **Last 15 minutes** (good for live view)
- **Last 1 hour** (more history)
- **Last 6 hours** (trends)

**Zoom**: Click and drag on any graph to zoom into time range

**Auto-refresh**: Should now refresh every 10 seconds automatically!

---

## Dashboard 1 Complete! ✅

### Success Checklist

- [ ] Dashboard shows 7 panels (4 stat + 3 time series)
- [ ] Temperature panel shows 2 lines (Node 1 and Node 2)
- [ ] Humidity panel shows 2 lines (Node 1 and Node 2) ✅ NEW!
- [ ] Gas resistance shows single green line (Node 1 only)
- [ ] All graphs are updating with live data
- [ ] Auto-refresh is working (see data update every 10s)
- [ ] Stat panels show current values with color coding
- [ ] Dashboard is saved (can navigate away and come back)

### What You Should See

| Metric | Node 1 | Node 2 | Notes |
|--------|--------|--------|-------|
| **Temperature** | ~28.4°C | ~27.9°C | 0.5°C difference (normal) |
| **Humidity** | ~51.7% | ~48.7% | Both SHT3x sensors working! ✅ |
| **Gas Resistance** | ~93kΩ | N/A | BME680 on Node 1 only |

---

# Part 3: Dashboard 2 - Signal Quality & Statistics

This dashboard monitors LoRa communication quality:
- RSSI (signal strength)
- SNR (signal-to-noise ratio)
- Packet statistics
- Error rates

---

## Step 1: Create New Dashboard (2 min)

1. Click **"+"** → **"Create Dashboard"**
2. Click **"Add visualization"**

---

## Step 2: Panel 1 - RSSI (Signal Strength) (5 min)

**Goal**: Show LoRa signal strength with color-coded quality

### Query

```flux
from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "rssi")
  |> filter(fn: (r) => r._field == "value")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
```

### Panel Settings

**Panel options:**
- **Title**: `LoRa Signal Strength (RSSI)`
- **Description**: `Received Signal Strength Indicator`

**Standard options:**
- **Unit**: `Signal > dBm`
- **Decimals**: `0`

**Thresholds** (color-coded signal quality):
- Base: Red (poor signal)
- Add: `-70` → Yellow (moderate)
- Add: `-50` → Green (good)
- Add: `-30` → Blue (excellent)

**Graph styles:**
- **Line width**: `2`
- **Fill opacity**: `20`
- **Gradient mode**: `Scheme`

Click **"Apply"**

**Expected**: Color-coded line showing -20 to -40 dBm (excellent signal, green/blue)

---

## Step 3: Panel 2 - SNR (Signal Quality) (4 min)

**Goal**: Show LoRa signal-to-noise ratio

### Query

```flux
from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "snr")
  |> filter(fn: (r) => r._field == "value")
  |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
```

### Panel Settings

**Panel options:**
- **Title**: `Signal Quality (SNR)`
- **Description**: `Signal-to-Noise Ratio - higher = cleaner signal`

**Standard options:**
- **Unit**: `Misc > Decibel (dB)`
- **Decimals**: `1`

**Thresholds**:
- Base: Red (poor)
- Add: `0` → Yellow
- Add: `5` → Light Green
- Add: `10` → Green (good)
- Add: `15` → Blue (excellent)

**Graph styles:**
- **Line width**: `2`
- **Fill opacity**: `20`
- **Gradient mode**: `Scheme`

Click **"Apply"**

**Expected**: Colored line showing ~12 dB (excellent, green)

---

## Step 4: Panel 3 - Packet Statistics (5 min)

**Goal**: Show received packets and errors as stat boxes

### Create Panel

1. **"Add"** → **"Visualization"**
2. **Change visualization type**: Select **"Stat"**
3. Select **"InfluxDB"** data source

### Query A - Packets Received

```flux
from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "packets_received")
  |> filter(fn: (r) => r._field == "value")
  |> last()
```

### Query B - CRC Errors

1. Click **"+ Query"**

```flux
from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "crc_errors")
  |> filter(fn: (r) => r._field == "value")
  |> last()
```

### Panel Settings

**Panel options:**
- **Title**: `Packet Statistics`
- **Description**: `Total packets received and CRC errors`

**Standard options:**
- **Unit**: `Short`

**Stat styles:**
- **Orientation**: `Horizontal`
- **Text mode**: `Value and name`
- **Color mode**: `Background`

**Value options:**
- **Show**: `Calculate` → `Last` (not null)

Click **"Apply"**

**Expected**: Two large stat boxes showing packet count and error count

---

## Step 5: Panel 4 - Error Rate (Advanced) (6 min)

**Goal**: Calculate and display packet error rate as percentage

### Query (Advanced - calculates percentage)

```flux
rx = from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "packets_received")
  |> aggregateWindow(every: v.windowPeriod, fn: last, createEmpty: false)

err = from(bucket: "telemetry")
  |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
  |> filter(fn: (r) => r._measurement == "crc_errors")
  |> aggregateWindow(every: v.windowPeriod, fn: last, createEmpty: false)

join(tables: {rx: rx, err: err}, on: ["_time"])
  |> map(fn: (r) => ({
      _time: r._time,
      _value: (float(v: r._value_err) / float(v: r._value_rx)) * 100.0
  }))
```

### Panel Settings

**Panel options:**
- **Title**: `Packet Error Rate`
- **Description**: `Percentage of packets with CRC errors`

**Visualization**: Select **"Gauge"** or **"Stat"**

**Standard options:**
- **Unit**: `Percent (0-100)`
- **Decimals**: `1`

**Thresholds**:
- Green: < 5% (excellent)
- Yellow: 5-15% (acceptable)
- Red: > 15% (poor)

Click **"Apply"**

**Note**: Your current error rate is ~50% (LoRa RF environment specific)

---

## Step 6: Arrange Layout & Save (5 min)

**Recommended layout:**

```
┌─────────────────────────────────────────────────────┐
│ Signal Quality & Statistics          [Auto-refresh] │
├─────────────────────────┬───────────────────────────┤
│ RSSI                    │ SNR                       │
│ (12 columns)            │ (12 columns)              │
├─────────────────────────┴───────────────────────────┤
│ Packet Statistics (Stat panel - full width)         │
│ Packets Received: 123   │   CRC Errors: 45          │
├──────────────────────────────────────────────────────┤
│ Packet Error Rate (Gauge - full width)              │
│                    [===50%===]                       │
└──────────────────────────────────────────────────────┘
```

### Save Dashboard

1. ⚙️ → **Settings**
2. **Name**: `Signal Quality & Statistics`
3. **Description**: `LoRa communication quality monitoring`
4. **Tags**: `iot`, `lora`, `signal`, `week8`
5. **Auto refresh**: `10s`
6. 💾 **Save dashboard**

---

## Dashboard 2 Complete! ✅

### Success Checklist

- [ ] Dashboard shows 4 panels (2 time series, 1 stat, 1 gauge)
- [ ] RSSI panel shows color-coded signal strength
- [ ] SNR panel shows signal quality with thresholds
- [ ] Packet statistics incrementing over time
- [ ] Error rate calculating correctly
- [ ] Auto-refresh working (10s)

---

# Part 4: Troubleshooting

## Panel shows "No data"

**Check:**
1. Time range includes recent data (use "Last 15 minutes")
2. Query is correct (copy from guide exactly)
3. Data source is selected (should say "InfluxDB")
4. Gateway is running: `docker compose ps`
5. Check logs: `docker compose logs -f gateway`

**Test query in Explore:**
1. Left sidebar → **Explore** (compass icon 🧭)
2. Select **InfluxDB** data source
3. Paste query
4. Click **Run query**
5. Should see data table/graph

**Verify data exists in InfluxDB:**
```bash
docker exec wk7-influxdb influx query \
  'from(bucket:"telemetry") |> range(start: -1h) |> limit(n:10)' \
  --org my-org --token my-super-secret-auth-token
```

---

## Query Error: "unknown measurement"

**Cause**: Typo in measurement name or no data yet

**Fix:**
- Check spelling (case-sensitive):
  - `temperature`
  - `humidity`
  - `gas_resistance`
  - `rssi`
  - `snr`
  - `packets_received`
  - `crc_errors`
- Verify data is being written (check gateway logs)
- Ensure time range is recent (Last 15 minutes)

---

## Graph looks choppy/jagged

**Cause**: Too many data points or not enough aggregation

**Fix**: Increase `windowPeriod` in query
```flux
|> aggregateWindow(every: 30s, fn: mean, createEmpty: false)
```

Change `30s` to `1m` or `5m` depending on time range.

---

## Colors don't show on RSSI/SNR

**Cause**: Thresholds not configured or gradient mode off

**Fix:**
1. Edit panel
2. Check **Thresholds** tab has values
3. Set **Graph styles > Gradient mode**: `Scheme`
4. Verify **Color mode**: `Thresholds`

---

## Data Source Connection Failed

**Error**: "Bad Gateway" or "Connection refused"

**Fix:**
1. Verify InfluxDB container is running:
   ```bash
   docker compose ps wk7-influxdb
   ```
2. Check URL uses container name: `http://wk7-influxdb:8086` (NOT localhost)
3. Verify token matches [docker-compose.yml](docker-compose.yml:43)
4. Test from Grafana container:
   ```bash
   docker exec wk7-grafana curl http://wk7-influxdb:8086/health
   ```

---

## Dashboard Not Auto-Refreshing

**Fix:**
1. Click ⚙️ Dashboard settings → **General**
2. Check **Auto refresh** has value (e.g., `10s`)
3. Verify **Refresh live dashboards** is enabled
4. Save dashboard
5. Refresh browser page

---

# Part 5: Useful Flux Query Patterns

## Get Last Value

```flux
from(bucket: "telemetry")
  |> range(start: -5m)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> last()
```

## Calculate Average Over Time

```flux
from(bucket: "telemetry")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> mean()
```

## Filter by Specific Node

```flux
from(bucket: "telemetry")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> filter(fn: (r) => r.node == "node1")  // Only Node 1
```

## Calculate Difference Between Nodes

```flux
n1 = from(bucket: "telemetry")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> filter(fn: (r) => r.node == "node1")

n2 = from(bucket: "telemetry")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "temperature")
  |> filter(fn: (r) => r.node == "node2")

join(tables: {n1: n1, n2: n2}, on: ["_time"])
  |> map(fn: (r) => ({
      _time: r._time,
      _value: r._value_n1 - r._value_n2
  }))
```

## Get Data Rate (Derivative)

```flux
from(bucket: "telemetry")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "packets_received")
  |> derivative(unit: 1m, nonNegative: true)
```

---

# Part 6: Available Measurements

Your InfluxDB `telemetry` bucket contains:

## From Node 1 (Remote Sensor):
- **`temperature`** (node=node1) - SHT3x temperature (°C)
- **`humidity`** (node=node1) - SHT3x humidity (%)
- **`gas_resistance`** (node=node1) - BME680 VOC sensor (Ω)

## From Node 2 (Gateway Local Sensor):
- **`temperature`** (node=node2) - SHT3x temperature (°C) ✅
- **`humidity`** (node=node2) - SHT3x humidity (%) ✅ **NEW!**

## Signal Quality:
- **`rssi`** - Received Signal Strength Indicator (dBm)
- **`snr`** - Signal-to-Noise Ratio (dB)

## Statistics:
- **`packets_received`** - Total packets received
- **`crc_errors`** - Total CRC errors

**All measurements** have:
- Field: `_field == "value"`
- Tags: `node` (node1, node2, signal, stats)
- Optional: `unit` (celsius, percent, ohms, etc.)

---

# Part 7: Dashboard Settings Reference

## Panel Sizing Tips

- **Stat panels** (current values): Height 4-5 units, width 6 columns
- **Time series graphs**: Height 8-10 units
- **Full-width graphs**: 24 grid columns
- **Half-width**: 12 grid columns each
- **Quarter-width**: 6 columns each

## Recommended Time Ranges

| Time Range | Use Case | Data Points |
|------------|----------|-------------|
| Last 5 minutes | Live monitoring | High density |
| Last 15 minutes | Recent trends | Good detail |
| Last 1 hour | Session overview | Balanced |
| Last 6 hours | Daily patterns | Aggregated |
| Last 24 hours | Full day view | Low density |

## Auto-Refresh Intervals

| Interval | Use Case | Load |
|----------|----------|------|
| 5s | Critical monitoring | High |
| 10s | Live dashboards (recommended) | Medium |
| 30s | Casual monitoring | Low |
| 1m | Background displays | Very low |

---

# Part 8: Export/Import Dashboards

## Export Dashboard (for backup)

1. Open dashboard
2. Dashboard settings (⚙️) → **JSON Model**
3. Copy JSON to clipboard
4. Save to file: `dashboard-environmental.json`

**Use cases:**
- Version control (commit to git)
- Backup before major changes
- Share with team members
- Deploy to other Grafana instances

## Import Dashboard

1. **+** → **Import**
2. Upload JSON file or paste JSON
3. Select **InfluxDB** data source
4. **Change UID** if duplicate exists
5. Click **Import**

---

# Part 9: Advanced Features (Optional)

## Variables for Flexibility

Create dashboard variables to make panels dynamic:

**Variable: `node`** (Node selector)
- Type: **Query**
- Query:
  ```flux
  from(bucket: "telemetry")
    |> range(start: -1h)
    |> keep(columns: ["node"])
    |> distinct(column: "node")
  ```
- Use in panels: `filter(fn: (r) => r.node == "${node}")`

**Variable: `interval`** (Time aggregation)
- Type: **Interval**
- Values: `10s,30s,1m,5m,15m`
- Use in panels: `aggregateWindow(every: ${interval}, fn: mean)`

## Alerts (Grafana 8+)

Set up alerts for critical conditions:

**Example: Temperature too high**

1. Edit panel → **Alert** tab
2. Create alert rule:
   - **Condition**: `WHEN last() OF query(A) IS ABOVE 35`
   - **For**: `5m` (must be true for 5 minutes)
3. Configure notification channel (email, Slack, Discord, etc.)
4. Add annotation to explain alert

**Example: Node offline**

Create alert for missing data:
- **Condition**: `WHEN last() OF query(A) HAS NO DATA`
- **For**: `2m`
- **Notification**: "Node 1 stopped transmitting!"

## Annotations for Events

Add event markers to graphs:

1. Dashboard settings → **Annotations**
2. Click **Add annotation query**
3. **Data source**: InfluxDB
4. **Query**:
   ```flux
   from(bucket: "events")
     |> range(start: v.timeRangeStart, stop: v.timeRangeStop)
   ```

**Use cases:**
- Deployment markers
- System restarts
- Configuration changes
- Firmware updates

---

# Part 10: Tips & Best Practices

## Query Performance

✅ **Do:**
- Use `aggregateWindow()` for downsampling
- Use `v.windowPeriod` for automatic resolution adjustment
- Limit time ranges for very dense data
- Use `last()` instead of aggregation for stat panels

❌ **Don't:**
- Query full time ranges without aggregation
- Use `aggregateWindow(every: 1s)` on large datasets
- Chain too many transformations

## Visual Design

✅ **Do:**
- Use consistent color scheme across panels
- Always specify units
- Add clear titles and descriptions
- Use thresholds for at-a-glance status
- Place legend where it doesn't block data

❌ **Don't:**
- Use too many colors (confusing)
- Omit units (ambiguous)
- Create panels without titles
- Use default "Panel Title"

## Dashboard Organization

✅ **Do:**
- Group related metrics together
- Put most important metrics at top
- Use consistent panel sizes
- Create logical flow (sensors → signal → stats)
- Use rows to organize sections

❌ **Don't:**
- Mix unrelated metrics
- Create cluttered layouts
- Use inconsistent sizing
- Overload single dashboard (split into multiple)

---

# Current Data Summary

Based on live telemetry (as of 2025-12-28):

| Metric | Node 1 | Node 2 | Notes |
|--------|--------|--------|-------|
| **Temperature** | 28.4°C | 27.9°C | 0.5°C difference (room temp) |
| **Humidity** | 51.7% | 48.7% | Both SHT3x sensors ✅ |
| **Gas Resistance** | 93kΩ | N/A | Node 1 BME680 only |
| **RSSI** | -26 to -40 dBm | - | Excellent signal quality |
| **SNR** | ~12 dB | - | Very good SNR |
| **Packets/min** | 2-3 | - | ~10s interval |
| **Error Rate** | ~50% | - | CRC errors (RF environment) |

---

# Next Steps

## Completed ✅

1. ✅ **Configure InfluxDB data source**
2. ✅ **Create Dashboard 1: Environmental Sensors**
3. ✅ **Create Dashboard 2: Signal Quality & Statistics**

## Suggested Next Steps

4. ⏭️ **Add alert rules** for critical thresholds
   - Temperature > 35°C
   - Humidity < 30% or > 70%
   - Node offline (no data for 2 minutes)

5. ⏭️ **Create Dashboard 3: System Health** (optional)
   - Data flow rate
   - Gateway uptime
   - InfluxDB write stats
   - MQTT message rate

6. ⏭️ **Add annotations** for events
   - Firmware updates
   - Configuration changes
   - System restarts

7. ⏭️ **Export dashboards** to JSON
   - Commit to version control
   - Backup before changes

8. ⏭️ **Mobile-friendly dashboard**
   - Simplified layout
   - Larger fonts
   - Fewer panels

---

# Resources

## Documentation

- **Flux Language**: https://docs.influxdata.com/flux/
- **Grafana Docs**: https://grafana.com/docs/grafana/latest/
- **InfluxDB v2 Query**: https://docs.influxdata.com/influxdb/v2/query-data/
- **Grafana Panel Plugins**: https://grafana.com/grafana/plugins/

## Your Project Files

- [docker-compose.yml](docker-compose.yml) - Infrastructure configuration
- [src/main.rs](src/main.rs) - Gateway service (MQTT/InfluxDB writes)
- [firmware/src/main.rs](firmware/src/main.rs) - Node 2 firmware (sensor reading)
- [SHT3X_SENSOR_FIX.md](SHT3X_SENSOR_FIX.md) - Sensor replacement documentation
- [WEEK_8_PROGRESS.md](WEEK_8_PROGRESS.md) - Progress tracking

---

# Success! 🎉

You now have:
- ✅ Professional IoT telemetry dashboards
- ✅ Real-time sensor monitoring
- ✅ Signal quality visualization
- ✅ Automatic refresh (10s)
- ✅ Color-coded thresholds
- ✅ Clean, organized layouts

**Total time**: 30-45 minutes
**Result**: Production-ready monitoring system!

Following the **baby steps philosophy**: Build one panel at a time, verify it works, then move to the next.

---

*Unified Grafana Dashboard Guide - Created: 2025-12-28*
*Includes SHT3x humidity data for both nodes!* ✅
