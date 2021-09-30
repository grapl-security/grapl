#!/bin/bash

set -euo pipefail

readonly NOMAD_LOGS_DEST=/tmp/nomad-agent.log
readonly CONSUL_LOGS_DEST=/tmp/consul-agent.log
THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR
cd "${THIS_DIR}"

VOLUME_CONFIG_FILE="mount-grapl-root-as-volume.nomad"
readonly VOLUME_CONFIG_FILE

ensure_valid_env() {
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

write_mount_grapl_root_volume_config() {
    # Every time we do a `nomad-agent &`, write a new file to
    # "per-invoke-nomad-conf.nomad"
    # that defines a Nomad host volume pointing at ${GRAPL_ROOT}
    # This is required for `python-integration-tests` to run Pants in an `exec` task.
    cat << EOF > "${VOLUME_CONFIG_FILE}"
# Written dynamically by 'write_mount_grapl_root_volume_config'
client {
  host_volume "grapl-root-volume" {
    path      = "${GRAPL_ROOT}"
    read_only = false
  }
}
EOF
}

start_nomad_detach() {
    ensure_valid_env
    write_mount_grapl_root_volume_config

    echo "Starting nomad and consul locally. Logs @ ${NOMAD_LOGS_DEST} and ${CONSUL_LOGS_DEST}."
    # These will run forever until `make stop-nomad-ci` is invoked."
    # shellcheck disable=SC2024
    sudo nomad agent \
        -config="nomad-agent-conf.nomad" \
        -config="${VOLUME_CONFIG_FILE}" \
        -dev-connect > "${NOMAD_LOGS_DEST}" &
    consul agent -dev > "${CONSUL_LOGS_DEST}" &

    ./nomad_run_local_infra.sh
    echo "Deployment complete"
}

start_nomad_detach
