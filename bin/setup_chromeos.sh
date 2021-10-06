#!/usr/bin/env bash

echo "Starting ChromeOS automated setup"

sudo apt update
sudo apt upgrade

echo "Fix bash completion"
sudo apt-get install --reinstall bash-completion

echo "Install build tooling"
sudo apt install -y build-essential libclang1

echo "Install docker"
curl -sSL https://get.docker.com/ | sh
sudo usermod -a -G docker "$USER"

echo "Installing rust toolchain"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

echo "Installing rust utilities"
cargo install -f ripgrep
cargo install -f fd-find
cargo install -f dua
cargo install -f bat

echo "Installing pyenv"

sudo apt-get install -y make libssl-dev zlib1g-dev libbz2-dev \
    libreadline-dev libsqlite3-dev wget curl llvm libncurses5-dev libncursesw5-dev \
    xz-utils tk-dev libffi-dev liblzma-dev
curl -L https://raw.githubusercontent.com/pyenv/pyenv-installer/master/bin/pyenv-installer | bash
pyenv install 3.7.10
pyenv global 3.7.10

echo "Installing nvm"
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.3/install.sh | bash
source "$HOME/.profile"
nvm install node

echo "Installing awscli"
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install
sudo rm awscliv2.zip

echo "Install pulumi"
curl -fsSL https://get.pulumi.com | sh

echo "Install useful utilities"
sudo apt-get install -y jq dnsutils

echo "Installing hashicorp tools: consul nomad packer"
curl -fsSL https://apt.releases.hashicorp.com/gpg | sudo apt-key add -
sudo apt-add-repository "deb [arch=amd64] https://apt.releases.hashicorp.com $(lsb_release -cs) main"
sudo apt-get update
sudo apt-get install -y consul nomad packer

get_latest_release() {
    curl --silent "https://api.github.com/repos/$1/releases/latest" | # Get latest release from GitHub api
        grep '"tag_name":' |                                          # Get tag line
        sed -E 's/.*"([^"]+)".*/\1/'                                  # Pluck JSON value
}

echo "Installing CNI plugins required for Nomad bridge networks"
sudo mkdir -p /opt/cni/bin
cd /opt/cni/bin || exit 2
VERSION=$(get_latest_release containernetworking/plugins)
sudo wget "https://github.com/containernetworking/plugins/releases/download/$VERSION/cni-plugins-linux-amd64-$VERSION.tgz"
curl -sL https://github.com/containernetworking/plugins/releases/latest | grep "linux-amd64" | grep -v "sha" | grep -v "span" | awk '{ print $2 }' | awk -F'"' '{ print "https://github.com"$2 }' | wget -qi -
sudo tar -xf "cni-plugins-linux-amd64-$VERSION.tgz"
sudo rm "cni-plugins-linux-amd64-$VERSION.tgz"

echo "Setting up workaround for Nomad linux fingerprinting bug"
echo "See https://github.com/hashicorp/nomad/issues/10902 for more context"
sudo mkdir -p "/lib/modules/$(uname -r)/"
echo '_/bridge.ko' | sudo tee -a /lib/modules/$(uname -r)/modules.builtin
