#!/usr/bin/env bash

set -euo pipefail

readonly GRAPL_ROOT="${PWD}"
readonly GRAPL_DEVBOX_DIR="${HOME}/.grapl_devbox"
mkdir -p "${GRAPL_DEVBOX_DIR}"

########################################
# Helper functions
########################################
has_key() {
    local -r input_json="${1}"
    local -r key="${2}"
    jq --exit-status ".[\"${key}\"]" <<< "${input_json}" > /dev/null
}

########################################
# Ensure we're in the Pulumi-enabled venv
########################################
if [ ! -f build-support/venv/bin/activate ]; then
    echo "Set up your virtualenv with 'build-support/manage_virtualenv.sh'"
    exit 42
fi
# shellcheck disable=SC1091
source build-support/venv/bin/activate

########################################
# Set up an SSH key specifically for the Devbox
########################################

SSH_PRIVATE_KEY_FILE="${GRAPL_DEVBOX_DIR}/ssh_key"
SSH_PUBLIC_KEY_FILE="${SSH_PRIVATE_KEY_FILE}.pub"

if [ ! -f "${SSH_PRIVATE_KEY_FILE}" ]; then
    ssh-keygen -t ed25519 -a 100 -C "${USER}@graplsecurity.com" -f "${SSH_PRIVATE_KEY_FILE}"
else
    echo "SSH Key already exists @ ${SSH_PRIVATE_KEY_FILE}"
fi

########################################
# Set up Pulumi stack
########################################

(
    cd "${GRAPL_ROOT}/devbox/provision/pulumi"
    STACK_NAME="grapl/${USER}-devbox"

    if ! pulumi stack --show-name --non-interactive; then
        pulumi stack init "${STACK_NAME}"
    else
        echo "Stack already exists"
    fi

    config=$(pulumi config --json)
    if ! has_key "${config}" "devbox:public-key}"; then
        pulumi config set devbox:public-key -- < "${SSH_PUBLIC_KEY_FILE}"
    fi
    if ! has_key "${config}" "devbox:private-key}"; then
        pulumi config set devbox:private-key --secret -- < "${SSH_PRIVATE_KEY_FILE}"
    fi
    if ! has_key "${config}" "devbox:instance-type}"; then
        # 32GB RAM
        # $5.80 daily reserved cost
        pulumi config set devbox:instance-type "m5.2xlarge"
    fi
    if ! has_key "${config}" "aws:region"; then
        echo "Hey there! You should go to devbox/provision/pulumi and run "
        echo "'pulumi config set aws:region <value>'"
        echo "Choose well - responsiveness is a genuine concern here!"
        echo "  ex: us-east-2, us-west-2, ap-east-1"
        exit 42
    fi
)

########################################
# Provision an EC2 instance with Pulumi
########################################

(
    cd "${GRAPL_ROOT}/devbox/provision/pulumi"
    pulumi update --refresh
)
