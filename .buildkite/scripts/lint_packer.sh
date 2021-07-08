#!/usr/bin/env bash
set -euo pipefail

echo "--- Linting Packer"
packer fmt -check -diff -recursive packer