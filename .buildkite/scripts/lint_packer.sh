#!/usr/bin/env bash
set -euo pipefail

echo "--- Linting Packer"
# Ideally we could do the following.
# packer fmt -check -diff -recursive packer/
packer fmt -check -diff packer/nomad-server-client/image.pkr.hcl
