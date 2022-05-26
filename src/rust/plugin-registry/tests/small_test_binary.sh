#!/usr/bin/env bash
echo "Hello, I am small_test_binary.sh"
echo "Starting a simple server on ${BIND_PORT}..."
apt-get update
apt-get install --yes python3
python3 -m http.server "${BIND_PORT}"
