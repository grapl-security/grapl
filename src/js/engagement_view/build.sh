#!/bin/bash
set -euo pipefail

yarn build
rm -rf ../grapl-cdk/edge_ux_pre_replace
cp -r ./build/. ../grapl-cdk/edge_ux_pre_replace
date
