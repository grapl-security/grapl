#!/bin/bash

set -euo pipefail

readonly NOMAD_LOGS_DEST=/tmp/nomad-agent.log
readonly CONSUL_LOGS_DEST=/tmp/consul-agent.log
readonly VAULT_LOGS_DEST=/tmp/vault-agent.log
THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR

ensure_cros_bridge_networking_workaround() {
    # Suggest the Nomad bridge networking hack in building.md if module is not found
    # Due to https://github.com/hashicorp/nomad/issues/10902

    # "is this crOS?" per
    # reddit.com/r/Crostini/comments/kuhwky/how_to_test_in_shell_script_whether_inside/girzz2y/
    if [[ -f /dev/.cros_milestone ]]; then
        module_filepath="/lib/modules/$(uname -r)/modules.builtin"
        if ! grep -q "_/bridge.ko" "$module_filepath"; then
            echo "It looks like you're on ChromeOS, but haven't installed the Nomad bridge networking workaround."
            # shellcheck source-path=SCRIPTDIR
            source "${THIS_DIR}/../../etc/chromeos/lib/installs.sh"
            (
                install_nomad_chromeos_workaround
            )
            echo "ChromeOS Nomad bridge networking workaround should now be installed. Continuing..."
        fi
    fi
}

ensure_firecracker_driver_installed() {
    expected_cni_path="/opt/nomad/plugins/firecracker-task-driver"
    if [[ ! -f "${expected_cni_path}" ]]; then
        echo "It looks like you don't have the Firecracker nomad stuff set up yet."
        # shellcheck source-path=SCRIPTDIR
        source "${THIS_DIR}/../../etc/chromeos/lib/installs.sh"
        (install_nomad_firecracker)
        echo "Continuing..."
    fi
}

ensure_valid_env() {
    ensure_cros_bridge_networking_workaround
    ensure_firecracker_driver_installed

    # ensure that all dependencies are available
    if [[ -z $(command -v nomad) ]]; then
        echo "Nomad must be installed. Please follow the install instructions at https://www.nomadproject.io/downloads"
        exit 2
    fi

    if [[ -z $(command -v consul) ]]; then
        echo "Consul must be installed. Please follow the install instructions at https://www.consul.io/downloads"
        exit 2
    fi
}

start_nomad_detach() {
    ensure_valid_env

    echo "Starting nomad, vault, and consul locally. Logs @ ${NOMAD_LOGS_DEST}, ${VAULT_LOGS_DEST} and ${CONSUL_LOGS_DEST}."
    # These will run forever until `make stop-nomad-ci` is invoked."
    # shellcheck disable=SC2024
    sudo nomad agent \
        -config="${THIS_DIR}/nomad-agent-conf.nomad" \
        -dev-connect > "${NOMAD_LOGS_DEST}" &

    vault server \
        -config="${THIS_DIR}/vault-agent-conf.hcl" \
        -dev > "${VAULT_LOGS_DEST}" &
    # The client is set to 0.0.0.0 here so that it can be reached via pulumi in docker.
    consul agent \
        -client 0.0.0.0 -config-file "${THIS_DIR}/consul-agent-conf.hcl" \
        -dev > "${CONSUL_LOGS_DEST}" &

    "${THIS_DIR}/nomad_run_local_infra.sh"
    echo "Deployment complete"
}

start_nomad_detach
