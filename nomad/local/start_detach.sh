#!/bin/bash

set -euo pipefail

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")"/../../src/sh/log.sh
# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")"/../../src/sh/dependencies.sh

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

ensure_valid_nomad_env() {
    ensure_cros_bridge_networking_workaround
    ensure_firecracker_driver_installed
}

configure_vault() {
    # We're using the root token for the POC of this
    VAULT_TOKEN=$(grep "Root Token" ${VAULT_LOGS_DEST} | awk '{ print $3 }')
    log_and_run vault secrets enable pki
    # enable intermediate pki
    log_and_run vault secrets enable -path=pki_int pki
}

create_dynamic_consul_config() {
    # clear file if it exist
    if [[ -f "${THIS_DIR}/consul-dynamic-conf.hcl" ]]; then
        rm "${THIS_DIR}/consul-dynamic-conf.hcl"
    fi

    GOSSIP_KEY=$(log_and_run consul keygen)

    # generate the file
    cat << EOF > "${THIS_DIR}/consul-dynamic-conf.hcl"
encrypt = "$GOSSIP_KEY"
connect {
  enabled = true
  ca_provider = "vault"
  ca_config {
    address = "http://127.0.0.1:8200"
    token = "$VAULT_TOKEN"
    root_pki_path = "connect-root"
    intermediate_pki_path = "connect-intermediate"
  }
}
EOF
}

clear_hashicorp_log_files() {
    declare -a HASHICORP_LOGS_DESTINATIONS=(CONSUL_LOGS_DEST NOMAD_LOGS_DEST VAULT_LOGS_DEST)
    for LOG_DEST in "${HASHICORP_LOGS_DESTINATIONS[@]}"; do
        if [[ -f "${LOG_DEST}" ]]; then
            rm "${LOG_DEST}"
        fi
    done
}

start_nomad_detach() {
    ensure_valid_nomad_env
    clear_hashicorp_log_files

    log "Starting nomad, vault, and consul locally. Logs @ ${NOMAD_LOGS_DEST}, ${VAULT_LOGS_DEST} and ${CONSUL_LOGS_DEST}."
    # Consul/Nomad/Vault  will run forever until `make down` is invoked."
    log_and_run vault server \
        -config="${THIS_DIR}/vault-agent-conf.hcl" \
        -dev > "${VAULT_LOGS_DEST}" 2>&1 &
    local -r vault_agent_pid="$!"

    # Wait for vault to boot
    export VAULT_ADDR="http://127.0.0.1:8200"
    (
        readonly attempts=15
        # shellcheck disable=SC2016
        timeout --foreground "${attempts}" bash -c -- "$(
            cat << EOF
                # We need to allow non-zero exit codes for vault status
                set +e
                # General rule: Variable defined in this EOF? Use \$
                wait_attempt=1
                # vault status returns an exit code of 0 for unsealed, 1 for error and 2 for sealed
                # Since we only want to capture the exit code, we need to drop all output from the command
                while  ! $(vault status &> /dev/null); do
                    if ! ps -p "${vault_agent_pid}" > /dev/null; then
                        echo "Vault Agent unexpectedly exited?"
                        exit 51
                    fi

                    echo "Waiting for vault to start [\${wait_attempt}/${attempts}]"
                    sleep 1
                    ((wait_attempt=wait_attempt+1))
                done
EOF
        )"
        # Vault is unsealed, but not yet initialized. We need to give it a sec to fully bootstrap
        sleep 2
    )

    configure_vault
    create_dynamic_consul_config

    # consul should be created prior to nomad to avoid a race condition
    # The client is set to 0.0.0.0 here so that it can be reached via pulumi in docker.
    log_and_run consul agent \
        -client 0.0.0.0 -config-file "${THIS_DIR}/consul-agent-conf.hcl" \
        -config-file "${THIS_DIR}/consul-dynamic-conf.hcl" \
        -dev > "${CONSUL_LOGS_DEST}" &
    local -r consul_agent_pid="$!"

    # Wait a short period of time before attempting to deploy infrastructure
    (
        readonly attempts=15
        # shellcheck disable=SC2016
        timeout --foreground "${attempts}" bash -c -- "$(
            cat << EOF
                # General rule: Variable defined in this EOF? Use \$
                set -euo pipefail
                wait_attempt=1
                while [[ -z \$(consul info 2>&1 | grep "leader = true") ]]; do
                    if ! ps -p "${consul_agent_pid}" > /dev/null; then
                        echo "Consul Agent unexpectedly exited?"
                        exit 51
                    fi

                    echo "Waiting for consul-agent [\${wait_attempt}/${attempts}]"
                    sleep 1
                    ((wait_attempt=wait_attempt+1))
                done
EOF
        )"
    )

    # shellcheck disable=SC2024
    log_and_run sudo nomad agent \
        -config="${THIS_DIR}/nomad-agent-conf.nomad" \
        -dev-connect > "${NOMAD_LOGS_DEST}" &
    local -r nomad_agent_pid="$!"

    # Ensure Nomad agent is ready
    (
        readonly attempts=30
        # shellcheck disable=SC2016
        timeout --foreground "${attempts}" bash -c -- "$(
            cat << EOF
                # General rule: Variable defined in this EOF? Use \$
                set -euo pipefail
                wait_attempt=1
                while [[ -z \$(nomad node status 2>&1 | grep ready) ]]; do
                    if ! ps -p "${nomad_agent_pid}" > /dev/null; then
                        echo "Nomad Agent unexpectedly exited?"
                        exit 51
                    fi

                    echo "Waiting for nomad-agent [\${wait_attempt}/${attempts}]"
                    sleep 1
                    ((wait_attempt=wait_attempt+1))
                done
                while [[ 2 -gt \$(consul info | grep service | awk '{ print \$3 }') ]]; do
                    echo "Waiting for consul-vault certs [\${wait_attempt}/${attempts}]"
                    sleep 1
                    ((wait_attempt=wait_attempt+1))
                done
EOF
        )"
    )

    "${THIS_DIR}/nomad_run_local_infra.sh"
    log "Deployment complete"
}

expect_binaries nomad consul vault
start_nomad_detach
