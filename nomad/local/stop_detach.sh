#!/bin/bash

set -euo pipefail

sudo killall nomad || true
killall consul || true
