#!/bin/bash
# Run the Week 7 MQTT + InfluxDB gateway service
# (spawns Node 2 firmware automatically via probe-rs)
clear
RUST_LOG=info cargo run
