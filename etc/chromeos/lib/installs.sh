#!/usr/bin/env bash

set -euo pipefail

# Set versions
PYENV_PYTHON_VERSION="3.7.10"

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

get_latest_release() {
    curl --silent "https://api.github.com/repos/$1/releases/latest" | # Get latest release from GitHub api
        grep '"tag_name":' |                                          # Get tag line
        sed -E 's/.*"([^"]+)".*/\1/'                                  # Pluck JSON value
}
## end helper functions

update_linux() {
    sudo apt update
    sudo apt upgrade
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
        libclang1
        lsb-release
        software-properties-common # for `apt-add-repository``
    )
    sudo apt install -y "${tools[@]}"
}

# potentially replace with podman in the future?
install_docker() {
    echo_banner "Install docker"
    curl -sSL https://get.docker.com/ | sh
    sudo usermod -a -G docker "$USER"

    echo_banner "Install docker-compose"
    sudo curl -L "https://github.com/docker/compose/releases/download/1.29.2/docker-compose-$(uname -s)-$(uname -m)" \
        -o /usr/local/bin/docker-compose
    sudo chmod +x /usr/local/bin/docker-compose
}

install_rust_and_utilities() {
    echo_banner "Installing rust toolchain"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    # Shellcheck can't follow $HOME or other vars like $USER so we disable the check here
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"

    echo_banner "Installing rust utilities (ripgrep, fd-find, dua and bat)"
    cargo install -f ripgrep
    cargo install -f fd-find
    cargo install -f dua
    cargo install -f bat
}

install_pyenv() {
    echo_banner "Install pyenv and set python version to ${PYENV_PYTHON_VERSION}"
    sudo apt-get install -y make libssl-dev zlib1g-dev libbz2-dev \
        libreadline-dev libsqlite3-dev wget curl llvm libncurses5-dev libncursesw5-dev \
        xz-utils tk-dev libffi-dev liblzma-dev

    # Check if pyenv directory exists and delete it so we can have a clean.

    # shellcheck disable=SC1091
    home_pyenv_dir="$HOME/.pyenv"
    if [ -d "${home_pyenv_dir}" ]; then
        echo ".pyenv already exists. Nuking it so that the pyenv is properly installed and configured"
        rm -rf "${home_pyenv_dir}"
    fi

    curl -L https://raw.githubusercontent.com/pyenv/pyenv-installer/master/bin/pyenv-installer | bash

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
    setup_pyenv_on_path
    pyenv install "${PYENV_PYTHON_VERSION}"
    pyenv global "${PYENV_PYTHON_VERSION}"
}

install_nvm() {
    echo_banner "Installing nvm"
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.3/install.sh | bash
    source_profile

    # Make nvm usable ASAP
    export NVM_DIR="${HOME}/.config/nvm"
    # shellcheck disable=SC1091
    [ -s "${NVM_DIR}/nvm.sh" ] && \. "${NVM_DIR}/nvm.sh" # This loads nvm
    # shellcheck disable=SC1091
    [ -s "${NVM_DIR}/bash_completion" ] && \. "${NVM_DIR}/bash_completion" # This loads nvm bash_completion

    nvm install node
}

install_awsv2() {
    echo_banner "Installing awscliv2"
    (
        cd /tmp
        curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
        unzip awscliv2.zip
        sudo ./aws/install --update
        sudo rm ./awscliv2.zip
        sudo rm -rf ./aws
    )
    echo_banner "Installing SSM extension"
    (
        cd /tmp
        curl "https://s3.amazonaws.com/session-manager-downloads/plugin/latest/ubuntu_64bit/session-manager-plugin.deb" \
            -o "session-manager-plugin.deb"
        sudo dpkg -i session-manager-plugin.deb
        rm ./session-manager-plugin.deb
    )
}
install_pulumi() {
    echo_banner "Install pulumi"
    curl -fsSL https://get.pulumi.com | sh
}

install_utilities() {
    echo_banner "Install useful utilities"
    sudo apt-get install -y jq dnsutils
}

install_hashicorp_tools() {
    echo_banner "Installing hashicorp tools: consul nomad packer"
    curl --proto '=https' --tlsv1.3 -sSf https://apt.releases.hashicorp.com/gpg |
        sudo gpg --no-default-keyring --keyring gnupg-ring:/etc/apt/trusted.gpg.d/hashicorp-apt.gpg --import &&
        sudo chmod 644 /etc/apt/trusted.gpg.d/hashicorp-apt.gpg
    sudo apt-add-repository "deb [arch=amd64] https://apt.releases.hashicorp.com $(lsb_release -cs) main"
    sudo apt-get update
    sudo apt-get install -y consul nomad packer
}

install_cni_plugins() {
    echo_banner "Installing CNI plugins required for Nomad bridge networks"
    sudo mkdir -p /opt/cni/bin
    cd /opt/cni/bin || exit 2
    VERSION=$(get_latest_release containernetworking/plugins)
    TGZ_NAME="cni-plugins-linux-amd64-${VERSION}.tgz"
    sudo wget "https://github.com/containernetworking/plugins/releases/download/${VERSION}/${TGZ_NAME}"
    sudo tar -xf "${TGZ_NAME}"
    sudo rm "${TGZ_NAME}"
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
