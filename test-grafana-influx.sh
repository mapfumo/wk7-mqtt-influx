#!/bin/bash
# Test Grafana + InfluxDB Integration
# Week 8 - Phase 2 Verification

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=================================================="
echo "Grafana + InfluxDB Integration Test"
echo "=================================================="
echo ""

# Test 1: Grafana Health
echo -n "1. Testing Grafana health... "
if curl -s http://localhost:3000/api/health | grep -q 'database.*ok'; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "   Grafana is not responding"
    exit 1
fi

# Test 2: InfluxDB Health
echo -n "2. Testing InfluxDB health... "
if curl -s http://localhost:8086/health | grep -q 'status.*pass'; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "   InfluxDB is not responding"
    exit 1
fi

# Test 3: InfluxDB from Grafana Container
echo -n "3. Testing InfluxDB from Grafana container... "
if docker exec wk7-grafana wget -q -O- http://wk7-influxdb:8086/health 2>/dev/null | grep -q 'status.*pass'; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "   Grafana container cannot reach InfluxDB"
    exit 1
fi

# Test 4: InfluxDB Data Availability
echo -n "4. Checking InfluxDB data... "
DATA_COUNT=$(docker exec wk7-influxdb influx query \
    'from(bucket:"telemetry") |> range(start: -1h) |> limit(n:1)' \
    --org my-org --token my-super-secret-auth-token 2>&1 | grep -c "_value" || true)

if [ "$DATA_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ OK${NC} (data available)"
else
    echo -e "${YELLOW}⚠ WARNING${NC} (no recent data)"
    echo "   Consider running: ./run-gateway.sh"
fi

# Test 5: Count Measurements
echo -n "5. Counting measurements... "
MEASUREMENTS=$(docker exec wk7-influxdb influx query \
    'import "influxdata/influxdb/schema"
     schema.measurements(bucket: "telemetry")' \
    --org my-org --token my-super-secret-auth-token 2>&1 | grep -E "temperature|humidity|pressure|rssi|snr|gas|packets|crc" | wc -l || echo "0")

if [ "$MEASUREMENTS" -ge 8 ]; then
    echo -e "${GREEN}✓ OK${NC} (found $MEASUREMENTS measurements)"
else
    echo -e "${YELLOW}⚠ WARNING${NC} (found $MEASUREMENTS measurements, expected 8)"
fi

echo ""
echo "=================================================="
echo "Summary"
echo "=================================================="
echo ""
echo -e "${GREEN}✓ Grafana is running and healthy${NC}"
echo -e "${GREEN}✓ InfluxDB is running and healthy${NC}"
echo -e "${GREEN}✓ Network connectivity verified${NC}"

if [ "$DATA_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Telemetry data is available${NC}"
else
    echo -e "${YELLOW}⚠ No recent data - start gateway to collect telemetry${NC}"
fi

echo ""
echo "Next Steps:"
echo "  1. Open Grafana: http://localhost:3000"
echo "  2. Login: admin / admin"
echo "  3. Add InfluxDB data source with these settings:"
echo "     - URL: http://wk7-influxdb:8086"
echo "     - Organization: my-org"
echo "     - Token: my-super-secret-auth-token"
echo "     - Default Bucket: telemetry"
echo "  4. Click 'Save & test'"
echo ""
echo "See GRAFANA_SETUP_GUIDE.md for detailed instructions"
echo ""
