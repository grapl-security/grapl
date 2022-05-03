#!/bin/bash

set -euo pipefail

sudo killall --wait nomad || true
killall --wait consul || true
killall --wait vault || true
