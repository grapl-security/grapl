#!/usr/bin/env bash

set -euo pipefail

# Set versions
PYENV_PYTHON_VERSION="3.7.10"

## helper functions
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
    echo "Fix bash completion"
    sudo apt-get install --reinstall bash-completion
}

install_build_tooling() {
    echo "Install build tooling"
    sudo apt install -y apt-utils build-essential libclang1
}

# potentially replace with podman in the future?
install_docker() {
    echo "Install docker"
    curl -sSL https://get.docker.com/ | sh
    sudo usermod -a -G docker "$USER"
}

install_rust_and_utilities() {
    echo "Installing rust toolchain"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    # Shellcheck can't follow $HOME or other vars like $USER so we disable the check here
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"

    echo "Installing rust utilities (ripgrep, fd-find, dua and bat)"
    cargo install -f ripgrep
    cargo install -f fd-find
    cargo install -f dua
    cargo install -f bat
}

install_pyenv() {
    echo "Install pyenv and set python version to ${PYENV_PYTHON_VERSION}"
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
        sed -Ei -e '/^([^#]|$)/ {a \
        export PYENV_ROOT="$HOME/.pyenv"
        a \
        export PATH="$PYENV_ROOT/bin:$PATH"
        a \
        ' -e ':a' -e '$!{n;ba};}' ~/.profile
        source ~/.profile
        echo 'eval "$(pyenv init --path)"' >>~/.profile
        echo 'eval "$(pyenv init -)"' >> ~/.bashrc
    }
    setup_pyenv_on_path
    pyenv install "${PYENV_PYTHON_VERSION}"
    pyenv global "${PYENV_PYTHON_VERSION}"
}

install_nvm() {
    echo "Installing nvm"
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.3/install.sh | bash
    # Shellcheck can't follow $HOME or other vars like $USER so we disable the check here
    # shellcheck disable=SC1091
    source "$HOME/.profile"

    # Make nvm usable ASAP
    export NVM_DIR="$HOME/.config/nvm"
    [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
    [ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion

    nvm install node
}

install_awsv2() {
    echo "Installing awscliv2"
    (
        cd /tmp
        curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
        unzip awscliv2.zip
        sudo ./aws/install --update
        sudo rm awscliv2.zip
    )
}
install_pulumi() {
    echo "Install pulumi"
    curl -fsSL https://get.pulumi.com | sh
}

install_utilities() {
    echo "Install useful utilities"
    sudo apt-get install -y jq dnsutils
}

install_hashicorp_tools() {
    echo "Installing hashicorp tools: consul nomad packer"
    curl -fsSL https://apt.releases.hashicorp.com/gpg | sudo apt-key add -
    sudo apt-get install -y lsb-release software-properties-common  # for `lsb_release` and `apt-add-repository``
    sudo apt-add-repository "deb [arch=amd64] https://apt.releases.hashicorp.com $(lsb_release -cs) main"
    sudo apt-get update
    sudo apt-get install -y consul nomad packer
}

install_cni_plugins() {
    echo "Installing CNI plugins required for Nomad bridge networks"
    sudo mkdir -p /opt/cni/bin
    cd /opt/cni/bin || exit 2
    VERSION=$(get_latest_release containernetworking/plugins)
    sudo wget "https://github.com/containernetworking/plugins/releases/download/$VERSION/cni-plugins-linux-amd64-$VERSION.tgz"
    curl -sL https://github.com/containernetworking/plugins/releases/latest | grep "linux-amd64" | grep -v "sha" | grep -v "span" | awk '{ print $2 }' | awk -F'"' '{ print "https://github.com"$2 }' | wget -qi -
    sudo tar -xf "cni-plugins-linux-amd64-$VERSION.tgz"
    sudo rm "cni-plugins-linux-amd64-$VERSION.tgz"
}

install_nomad_chromeos_workaround() {
    echo "Setting up workaround for Nomad linux fingerprinting bug"
    echo "See https://github.com/hashicorp/nomad/issues/10902 for more context"
    sudo mkdir -p "/lib/modules/$(uname -r)/"
    echo '_/bridge.ko' | sudo tee -a "/lib/modules/$(uname -r)/modules.builtin"
}

install_git_hooks() {
    echo "Installing git hooks"
    GIT_ROOT=$(git rev-parse --show-toplevel)
    ln --symbolic --relative --force "$GIT_ROOT/etc/hooks/pre-commit.sh" "$GIT_ROOT/.git/hooks/pre-commit"
}
