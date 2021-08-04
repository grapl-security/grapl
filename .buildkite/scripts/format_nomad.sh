#!/usr/bin/env bash
set -euo pipefail

# Nomad doesn't have a `fmt` command but packer's works
# Packer, however, won't pick up nomad files (even if their extension is .hcl)
# so we're going to pass in each nomad file, individually

find nomad/ -type f -name "*.nomad" | xargs -n 1 packer fmt 