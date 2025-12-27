#!/bin/bash
# Subscribe to MQTT test messages using Docker

echo "Subscribing to iiot/# topics..."
echo "Press Ctrl+C to stop"
echo ""

docker run --rm -it --network wk7-mqtt-influx_iiot-network eclipse-mosquitto:2 \
    mosquitto_sub -h wk7-mosquitto -t "iiot/#" -v
