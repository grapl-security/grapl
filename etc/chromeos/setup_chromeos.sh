#!/usr/bin/env bash
# This is a WIP set up script for chromeOS. Some commands may break by the time the script is next used
# TODO ensure idempotency

source etc/chromeos/lib/installs.sh

echo "Starting ChromeOS automated setup"
update_linux
fix_shell_completion
install_build_tooling
install_docker
install_rust_and_utilities
nuke_home_venv
install_pyenv
install_nvm
install_awsv2
install_pulumi
install_utilities
install_hashicorp_tools
install_cni_plugins
install_nomad_chromeos_workaround
