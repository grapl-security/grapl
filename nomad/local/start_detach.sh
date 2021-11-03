#!/bin/bash

set -euo pipefail

readonly NOMAD_LOGS_DEST=/tmp/nomad-agent.log
readonly CONSUL_LOGS_DEST=/tmp/consul-agent.log
THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR
cd "${THIS_DIR}"

ensure_cros_bridge_networking_workaround() {
    # Suggest the Noamd bridge networking hack in building.md if module is not found
    # Due to https://github.com/hashicorp/nomad/issues/10902

    # "is this crOS?" per
    # reddit.com/r/Crostini/comments/kuhwky/how_to_test_in_shell_script_whether_inside/girzz2y/
    if [[ -f /dev/.cros_milestone ]]; then
        module_filepath="/lib/modules/$(uname -r)/modules.builtin"
        if ! grep -q "_/bridge.ko" "$module_filepath"; then
            echo "It looks like you're on ChromeOS, but haven't installed the Nomad bridge networking workaround."
            source etc/chromeos/lib/installs.sh
            install_nomad_chromeos_workaround
            echo "ChromeOS Nomad bridge networking workaround should now be installed. Continuing..."
        fi
    fi
}

ensure_valid_env() {
    ensure_cros_bridge_networking_workaround

    # Ensure script is being run with `local-grapl.env` variables
    # via `make start-nomad-ci`
    if [[ ! -v DOCKER_REGISTRY ]]; then
        echo "!!! Run this with 'make start-nomad-ci'"
        exit 1
    fi

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

    echo "Starting nomad and consul locally. Logs @ ${NOMAD_LOGS_DEST} and ${CONSUL_LOGS_DEST}."
    # These will run forever until `make stop-nomad-ci` is invoked."
    # shellcheck disable=SC2024
    sudo nomad agent \
        -config="nomad-agent-conf.nomad" \
        -dev-connect > "${NOMAD_LOGS_DEST}" &
    consul agent -dev > "${CONSUL_LOGS_DEST}" &

    ./nomad_run_local_infra.sh
    echo "Deployment complete"
}

start_nomad_detach
