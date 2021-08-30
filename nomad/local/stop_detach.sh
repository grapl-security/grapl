#!/bin/bash

set -euo pipefail

killall nomad || true
killall consul || true