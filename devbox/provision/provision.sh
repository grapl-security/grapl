#!/usr/bin/env bash

set -euo pipefail

readonly GRAPL_ROOT="${PWD}"
THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/../lib.sh"

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

readonly SSH_PRIVATE_KEY_FILE="${GRAPL_DEVBOX_DIR}/devbox_ed25519_ssh"
readonly SSH_PUBLIC_KEY_FILE="${SSH_PRIVATE_KEY_FILE}.pub"

if [ ! -f "${SSH_PRIVATE_KEY_FILE}" ]; then
    ssh-keygen -t ed25519 -a 100 -C "${USER}@graplsecurity.com" -f "${SSH_PRIVATE_KEY_FILE}"
else
    echo "SSH Key already exists @ ${SSH_PRIVATE_KEY_FILE}"
fi

########################################
# Tell SSH to use SSM trickery on hosts starting with `i-`
########################################

(
    # Taken from https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-getting-started-enable-ssh-connections.html
    SSH_CONFIG_APPEND="$(
        cat << 'EOF'
host i-* mi-*
    ProxyCommand sh -c "aws ssm start-session --target %h --document-name AWS-StartSSHSession --parameters 'portNumber=%p'"
EOF
    )"
    touch ~/.ssh/config
    if ! grep --quiet "${SSH_CONFIG_APPEND}" ~/.ssh/config; then
        echo "${SSH_CONFIG_APPEND}" >> ~/.ssh/config
    fi
)

########################################
# Set up Pulumi stack
########################################

(
    cd "${GRAPL_ROOT}/devbox/provision"
    STACK_NAME="grapl/${USER}-devbox"

    if ! pulumi stack --show-name --non-interactive; then
        pulumi stack init "${STACK_NAME}"
    else
        echo "Stack ${STACK_NAME} already exists"
    fi

    config=$(pulumi config --json)
    if ! has_key "${config}" "devbox:public-key}"; then
        pulumi config set devbox:public-key -- < "${SSH_PUBLIC_KEY_FILE}"
    fi
    if ! has_key "${config}" "devbox:instance-volume-size-gb}"; then
        pulumi config set devbox:instance-volume-size-gb 100
    fi
    if ! has_key "${config}" "devbox:instance-type}"; then
        # 32GB RAM
        # $5.80 daily reserved cost
        pulumi config set devbox:instance-type "m5.2xlarge"
    fi
    if ! has_key "${config}" "aws:region"; then
        echo "Hey there! You should go to '$(pwd)' and run "
        echo "'pulumi config set aws:region <value>'"
        echo "Choose well - responsiveness is a genuine concern here!"
        echo "  ex: us-east-2, us-west-2, ap-east-1"
        exit 42
    fi
)

########################################
# Provision an EC2 instance with Pulumi
########################################
pulumi update --yes --cwd="${GRAPL_ROOT}/devbox/provision"

########################################
# Copy some config stuff to a .env file consumed by ssh.sh
########################################
(
    cd "${GRAPL_ROOT}/devbox/provision"

    ENV_CONFIG="$(
        cat << EOF
GRAPL_DEVBOX_REGION="$(pulumi config get aws:region)"
GRAPL_DEVBOX_INSTANCE_ID="$(pulumi stack output devbox-instance-id)"
GRAPL_DEVBOX_USER="$(pulumi stack output devbox-user)"
GRAPL_DEVBOX_PRIVATE_KEY_FILE="${SSH_PRIVATE_KEY_FILE}"
GRAPL_DEVBOX_REMOTE_REPOS="/home/$(pulumi stack output devbox-user)/repos"
GRAPL_DEVBOX_REMOTE_GRAPL="/home/$(pulumi stack output devbox-user)/repos/grapl"
GRAPL_DEVBOX_LOCAL_GRAPL="${GRAPL_ROOT}"
EOF
    )"
    echo "${ENV_CONFIG}" > "${GRAPL_DEVBOX_CONFIG}"
)

# Bring these environment variables into the current shell
# shellcheck disable=SC1090
source "${GRAPL_DEVBOX_CONFIG}"

########################################
# One-time changes to the box
########################################
(
    CMD="$(
        cat << EOF
    if [ ! -d "${GRAPL_DEVBOX_REMOTE_GRAPL}" ]; then
        echo "Checking out Grapl repo on remote"
        mkdir -p "${GRAPL_DEVBOX_REMOTE_REPOS}"
        cd "${GRAPL_DEVBOX_REMOTE_REPOS}"
        # Gotta do the https because our ssh key can't read from github
        git clone https://github.com/grapl-security/grapl.git
        cd grapl
        ./etc/chromeos/setup_chromeos.sh
    fi
EOF
    )"
    # There's a minor race condition here, where Pulumi starts the box but it's
    # not ready to take SSH-over-SSM commands quite yet.
    "${THIS_DIR}/../ssh.sh" -- "${CMD}"
)
