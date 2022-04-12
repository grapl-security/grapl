#!/usr/bin/env bash
# This script installs optional tooling to improve development.

set -euo pipefail

source etc/chromeos/lib/installs.sh

echo "Starting Grapl OPTIONAL tooling installation"
install_git_hooks
