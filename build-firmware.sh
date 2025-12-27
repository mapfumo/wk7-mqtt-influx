#!/bin/bash
# Build Node 2 gateway firmware
clear
cd firmware && \
cargo build --release --target thumbv7em-none-eabihf
