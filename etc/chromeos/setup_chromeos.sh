#!/usr/bin/env bash
# This is a WIP set up script for chromeOS. Some commands may break by the time the script is next used
# TODO ensure idempotency

set -euo pipefail

source etc/chromeos/lib/installs.sh

echo "Starting ChromeOS automated setup"

configure_grapl_repository

update_linux
fix_shell_completion
install_build_tooling
install_utilities
install_protoc
install_rust_and_utilities
install_pyenv
install_pipx
install_nvm
install_awsv2
install_pulumi
install_hashicorp_tools
install_cni_plugins
install_firecracker
install_nomad_chromeos_workaround
install_nomad_firecracker
install_sqlx_prepare_deps
install_bk
install_docker # Do this last since it's not idempotent
