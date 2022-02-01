#!/bin/bash

set -euo pipefail

sudo killall --wait nomad || true
killall --wait consul || true
docker volume rm local-grapl-zk-data --force
docker volume rm local-grapl-zk-txn-logs --force
docker volume rm local-grapl-kafka-data --force
