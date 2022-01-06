#!/usr/bin/env bash
# This is a WIP set up script for chromeOS. Some commands may break by the time the script is next used
# TODO ensure idempotency

set -euo pipefail

source etc/chromeos/lib/installs.sh

echo "Starting ChromeOS automated setup"
update_linux
fix_shell_completion
install_build_tooling
install_rust_and_utilities
install_pyenv
install_nvm
install_awsv2
install_pulumi
install_utilities
install_hashicorp_tools
install_cni_plugins
install_nomad_chromeos_workaround
install_git_hooks
install_docker # Do this last since it's not idempotent
