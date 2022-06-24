#!/usr/bin/env bash

set -euo pipefail

# We're starting to use this  script for more than chromebooks. As such we're starting to make this
# architecture-independent, so that in the future we can use it for AWS graviton instances, which are significantly
# more cost-effective, especially the metal ones.
# As such this section sets up some architecture variables..
ARCH=$(arch)
if [ "${ARCH}" == "x86_64" ]; then
    ssm_arch_alias="64bit"
else
    ssm_arch_alias="arm64"
fi

## helper functions
source_profile() {
    # Shellcheck can't follow $HOME or other vars like $USER so we disable the check here
    # shellcheck disable=SC1091
    source "$HOME/.profile"
}

echo_banner() {
    echo -e "\n\n========================================"
    echo "==> ${1} "
    echo -e "========================================\n"
}

should_force_reinstall() {
    # One flag to represent "let's overwrite all existing installs"
    if [[ -n "${FORCE_REINSTALL:-}" ]]; then
        true
    else
        false
    fi
}

_cargo_install() {
    declare -a cargo_install_flags=()
    if should_force_reinstall; then
        cargo_install_flags+=("--force")
    fi

    echo "Cargo Install:" "${@}"
    echo "(Set FORCE_REINSTALL=1 to force install)"
    cargo install "${cargo_install_flags[@]}" "${@}"
}

get_latest_release() {
    curl --proto "=https" \
        --tlsv1.2 \
        --silent \
        "https://api.github.com/repos/$1/releases/latest" |
        jq --raw-output '.tag_name'
}
## end helper functions

configure_grapl_repository() {
    echo_banner "Setting up Grapl Repository"
    curl --proto "=https" \
        --tlsv1.2 \
        --location \
        --fail \
        --silent \
        "https://dl.cloudsmith.io/public/grapl/deb/setup.deb.sh" |
        sudo -E bash
}

update_linux() {
    sudo apt update
    sudo apt upgrade --yes
}

fix_shell_completion() {
    # TODO add support for other shells like zsh
    echo_banner "Fix bash completion"
    sudo apt-get install --reinstall bash-completion
}

install_build_tooling() {
    echo_banner "Install build tooling"
    tools=(
        apt-utils
        build-essential
        cmake # necessary for building rust-rdkafka
        libclang1
        lsb-release
        software-properties-common # for `apt-add-repository``
    )
    sudo apt install --yes "${tools[@]}"
}

# potentially replace with podman in the future?
install_docker() {
    echo_banner "Install docker"
    if (! command -v docker) || should_force_reinstall; then
        curl --proto "=https" \
            --tlsv1.2 \
            --silent \
            --show-error \
            --location \
            https://get.docker.com/ | sh
        sudo usermod -a -G docker "$USER"
    fi

    echo_banner "Install docker compose (v2, new, Go) CLI plugin"
    user_docker_cli_plugins_dir="${HOME}/.docker/cli-plugins"
    mkdir --parents "${user_docker_cli_plugins_dir}"

    curl --proto "=https" \
        --tlsv1.2 \
        --location \
        --output "${user_docker_cli_plugins_dir}/docker-compose" \
        "https://github.com/docker/compose/releases/download/v2.2.3/docker-compose-$(uname --kernel-name | tr '[:upper:]' '[:lower:]')-$(uname --machine)"
    chmod +x "${user_docker_cli_plugins_dir}/docker-compose"
}

install_rust_and_utilities() {
    if (! command -v rustup) || should_force_reinstall; then
        echo_banner "Installing rust toolchain"
        # -y means "disable confirmation prompt". No, there's no --yes
        curl --proto "=https" \
            --tlsv1.2 \
            --silent \
            --show-error \
            --fail https://sh.rustup.rs | sh -s -- -y
    fi
    # Shellcheck can't follow $HOME or other vars like $USER so we disable the check here
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"

    rust_utilities=(
        ripgrep
        fd-find
        dua
        bat
    )

    echo_banner "Installing rust utilities"

    _cargo_install "${rust_utilities[@]}"
}

install_pyenv() {
    echo_banner "Install pyenv"
    sudo apt-get install --yes make libssl-dev zlib1g-dev libbz2-dev \
        libreadline-dev libsqlite3-dev wget curl llvm libncurses5-dev libncursesw5-dev \
        xz-utils tk-dev libffi-dev liblzma-dev python3-dev

    # Check if pyenv directory exists and delete it so we can have a clean.

    # shellcheck disable=SC1091
    home_pyenv_dir="$HOME/.pyenv"
    if [ -d "${home_pyenv_dir}" ] && should_force_reinstall; then
        echo ".pyenv already exists. Nuking it so that the pyenv is properly installed and configured"
        rm -rf "${home_pyenv_dir}"
    fi

    # This function is stolen from the
    # "If your ~/.profile sources ~/.bashrc (Debian, Ubuntu, Mint)"
    # section of https://github.com/pyenv/pyenv/blob/master/README.md
    setup_pyenv_on_path() {
        # the sed invocation inserts the lines at the start of the file
        # after any initial comment lines
        # shellcheck disable=0-9999
        sed -Ei -e '/^([^#]|$)/ {a \
        export PYENV_ROOT="$HOME/.pyenv"
        a \
        export PATH="$PYENV_ROOT/bin:$PATH"
        a \
        ' -e ':a' -e '$!{n;ba};}' ~/.profile

        source_profile
        # shellcheck disable=SC2016
        echo 'eval "$(pyenv init --path)"' >> ~/.profile
        # shellcheck disable=SC2016
        echo 'eval "$(pyenv init -)"' >> ~/.bashrc
    }

    if [ ! -d "${home_pyenv_dir}" ]; then
        curl --proto "=https" \
            --tlsv1.2 \
            --location \
            https://raw.githubusercontent.com/pyenv/pyenv-installer/master/bin/pyenv-installer | bash
        setup_pyenv_on_path
    fi

    source_profile
    pyenv install --skip-existing
    # Sets global Python to the same thing that is configured in
    # .python-version in this repository
    pyenv global "$(pyenv local)"
}

install_pipx() {
    echo_banner "Installing pipx"
    python3 -m pip install --user pipx --upgrade
    python3 -m pipx ensurepath
}

install_nvm() {
    echo_banner "Installing nvm"
    # This exported variable is actually used by the NVM install script
    export NVM_DIR="${HOME}/.config/nvm"
    mkdir --parents "${NVM_DIR}"

    curl --proto "=https" \
        --tlsv1.2 \
        https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.1/install.sh | bash
    source_profile

    # shellcheck disable=SC1091
    [ -s "${NVM_DIR}/nvm.sh" ] && \. "${NVM_DIR}/nvm.sh" # This loads nvm
    # shellcheck disable=SC1091
    [ -s "${NVM_DIR}/bash_completion" ] && \. "${NVM_DIR}/bash_completion" # This loads nvm bash_completion

    # Install latest node 16.x. This matches up with engagement_view, although graphql_endpoint is on 17 :(
    nvm install 16
    # Opt in to corepack. With this on, we'll use the version of yarn set by the packageManager property in package.json
    # Yes, with this on we'll have one source of truth for yarn versions!
    corepack enable
}

install_awsv2() {
    if (! command -v aws) || should_force_reinstall; then
        echo_banner "Installing awscliv2"
        (
            cd /tmp
            curl --proto "=https" \
                --tlsv1.2 \
                --output "awscliv2.zip" \
                "https://awscli.amazonaws.com/awscli-exe-linux-${ARCH}.zip"
            unzip awscliv2.zip
            sudo ./aws/install --update
            sudo rm ./awscliv2.zip
            sudo rm -rf ./aws
        )
        echo_banner "Installing SSM extension"
        (
            cd /tmp
            curl --proto "=https" \
                --tlsv1.2 \
                --remote-name \
                "https://s3.amazonaws.com/session-manager-downloads/plugin/latest/ubuntu_${ssm_arch_alias}/session-manager-plugin.deb"
            sudo dpkg -i session-manager-plugin.deb
            rm ./session-manager-plugin.deb
        )
    fi
}
install_pulumi() {
    echo_banner "Install pulumi"
    curl --proto "=https" \
        --tlsv1.2 \
        --fail \
        --silent \
        --show-error \
        --location \
        https://get.pulumi.com | sh
}

install_utilities() {
    echo_banner "Install useful utilities"
    sudo apt-get install --yes jq dnsutils tree unzip rsync
}

install_hashicorp_tools() {
    echo_banner "Installing hashicorp tools: consul nomad packer"

    # Set specific versions since we're enabling the hashicorp test repo
    CONSUL_VERSION="1.12.2-1"
    NOMAD_VERSION="1.3.1-1"
    # packer doesn't have the -1s at the end for some reason, until 1.8.1. When upgrading please confirm with `apt-cache showpkg packer`
    PACKER_VERSION="1.8.1-1"
    VAULT_VERSION="1.10.3-1"

    sudo apt-get install --yes --allow-downgrades \
        consul="${CONSUL_VERSION}" \
        nomad="${NOMAD_VERSION}" \
        packer="${PACKER_VERSION}" \
        vault="${VAULT_VERSION}"
}

# Download and install a tarball.
#
# Assumptions:
# - URL is HTTPS
# - URL is for a tar.gz file
# - Target directory must be created / written to with root
#   permissions
# - The tarball has all its contents at the root of the
#   archive. Everything in the archive will be moved as-is into the
#   destination directory.
# - All the things in the tarball will be given 755 permissions, and
#   will be owned by root:root
download_and_install_tarball() {
    local -r url="${1}"
    local -r target_dir="${2}"

    file_name="$(basename "${url}")"

    # Retrieve the archive
    curl --proto "=https" \
        --tlsv1.2 \
        --location \
        --remote-name \
        "${url}"

    # Create a dedicated temporary directory to store the extracted
    # contents of the tarball, prior to moving it.
    temp_dir="$(mktemp --directory)"

    # Extract the archive into the temporary directory
    tar --extract \
        --verbose \
        --directory="${temp_dir}" \
        --file="${file_name}"

    # The permissions for the firecracker-task-driver currently come
    # as 775, rather than 755; all the CNI plugins are already
    # 755. This just takes care of all of them at once
    chmod --recursive --verbose 0755 "${temp_dir}"/*

    # Create the destination and move everything to it.
    #
    # These are the only commands that need sudo privileges.
    sudo mkdir --parents "${target_dir}"
    sudo chown root:root "${temp_dir}"/*
    sudo mv "${temp_dir}"/* "${target_dir}"

    # Show the contents of the target_dir for visibility and debugging
    tree "${target_dir}"

    # Clean up after ourselves
    rm "${file_name}"
    rm -Rf "${temp_dir}"
}

install_cni_plugins() {
    echo_banner "Installing CNI plugins required for Nomad bridge networks"

    repo="containernetworking/plugins"
    version="$(get_latest_release "${repo}")"

    download_and_install_tarball \
        "https://github.com/${repo}/releases/download/${version}/cni-plugins-linux-amd64-${version}.tgz" \
        /opt/cni/bin
}

install_firecracker() {
    echo_banner "Installing Firecracker binary"

    repo="firecracker-microvm/firecracker"
    # v1.0.0 doesn't currently work with the nomad firecracker plugin due to a breaking change. Instead we're hardcoding
    # the version for now. TODO switch to grabbing the latest version once the nomad plugin is updated
    # version=$(get_latest_release "${repo}")
    version="v0.18.0"

    url_prefix="https://github.com/${repo}/releases/download/${version}"

    # This is used for old versions of firecracker
    sudo curl --proto "=https" \
        --tlsv1.2 \
        --location \
        --output /usr/bin/firecracker \
        "${url_prefix}/firecracker-${version}"
    sudo chmod 0755 /usr/bin/firecracker

    # TODO switch to grabbing the tarball release once the task driver is upgraded
    #    download_and_install_tarball \
    #        "${url_prefix}/firecracker-${version}-${ARCH}.tgz" \
    #        /tmp/firecracker
    #
    #    sudo mv "/tmp/firecracker/release-${version}-${ARCH}/firecracker-${version}-${ARCH}" "/usr/bin/firecracker"
}

install_nomad_firecracker() {
    echo_banner "Installing Firecracker Nomad driver and dependencies"

    repo="cneira/firecracker-task-driver"
    version=$(get_latest_release "${repo}")

    url_prefix="https://github.com/${repo}/releases/download/${version}"

    download_and_install_tarball \
        "${url_prefix}/firecracker-task-driver-${version}.tar.gz" \
        /opt/nomad/plugins

    sudo curl --proto "=https" \
        --tlsv1.2 \
        --location \
        --output /opt/cni/bin/tc-redirect-tap \
        "https://dl.cloudsmith.io/public/grapl/thomas/raw/files/tc-redirect-tap"

    sudo chmod 0755 /opt/cni/bin/tc-redirect-tap

}

install_nomad_chromeos_workaround() {
    echo_banner "Setting up workaround for Nomad linux fingerprinting bug"
    echo "See https://github.com/hashicorp/nomad/issues/10902 for more context"
    sudo mkdir -p "/lib/modules/$(uname -r)/"
    echo '_/bridge.ko' | sudo tee -a "/lib/modules/$(uname -r)/modules.builtin"
}

install_git_hooks() {
    echo_banner "Installing git hooks"
    GIT_ROOT=$(git rev-parse --show-toplevel)
    ln --symbolic --relative --force "$GIT_ROOT/etc/hooks/pre-commit.sh" "$GIT_ROOT/.git/hooks/pre-commit"
}

install_sqlx_prepare_deps() {
    _cargo_install sqlx-cli --no-default-features --features postgres,rustls
    sudo apt install --yes netcat # used for `nc`
}
