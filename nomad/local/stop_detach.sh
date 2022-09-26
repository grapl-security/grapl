#!/bin/bash

set -euo pipefail

#sudo systemctl stop nomad || true
sudo killall --wait nomad || true
killall --wait consul || true
killall --wait vault || true
