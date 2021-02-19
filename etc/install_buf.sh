#!/usr/bin/env bash

set -euo pipefail

# Downloads the `buf` Protobuf tooling binary (https://buf.build/)
# from Github Releases to the current directory.

# See https://github.com/bufbuild/buf/releases for latest version
version=0.36.0

os=$(uname --kernel-name)
arch=$(uname --machine)

curl --silent \
     --show-error \
     --location \
     "https://github.com/bufbuild/buf/releases/download/v${version}/buf-${os}-${arch}" \
     --output buf

chmod +x buf

>&2 echo "Installation of 'buf' version ${version} to the current directory is complete"
