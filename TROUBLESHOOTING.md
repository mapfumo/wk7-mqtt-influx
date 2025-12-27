# Week 7 Troubleshooting Guide

Common issues and solutions for MQTT + InfluxDB integration.

---

## Docker Issues

### Error: `permission denied while trying to connect to Docker daemon`

**Symptom**:
```
permission denied while trying to connect to the Docker daemon socket
```

**Cause**: User not in `docker` group or group membership not active

**Fix**:
```bash
# Check groups
groups

# Add to docker group
sudo usermod -aG docker $USER

# Apply (choose one):
newgrp docker          # New shell
# Or log out/in
```

---

### Error: `network not found`

**Cause**: Docker services not running

**Fix**:
```bash
docker compose ps
docker compose up -d
```

---

## MQTT Issues

### Error: `Connection refused`

**Diagnosis**:
```bash
# Check Mosquitto running
docker compose ps
docker compose logs mosquitto

# Test connection
docker run --rm --network wk7-mqtt-influx_iiot-network \
    eclipse-mosquitto:2 mosquitto_pub -h wk7-mosquitto -t test -m "hello"
```

---

### Issue: No messages appearing

**Check**:
1. Right topic? Use `iiot/#` for all
2. Right network? Use `wk7-mqtt-influx_iiot-network`
3. Gateway logs show "Published to MQTT"?

---

## InfluxDB Issues

### Error: `unauthorized access`

**Fix**: Check token in config.toml matches Docker init:
```bash
cat config.toml | grep token
# Should be: my-super-secret-auth-token
```

---

### Error: `bucket not found`

**Fix**: Recreate with initialization:
```bash
docker compose down -v
docker compose up -d
```

---

## Useful Commands

```bash
# Check services
docker compose ps

# View logs
docker compose logs mosquitto
docker compose logs influxdb

# Subscribe to MQTT
./test-mqtt-sub.sh

# Reset everything
docker compose down -v
docker compose up -d
```

---

*Last Updated*: 2025-12-28
