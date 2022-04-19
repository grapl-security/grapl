#!/bin/bash

set -euo pipefail

# check systemd version
SYSTEMD_VERSION=$(systemctl --version | head --lines 1 | awk '{ print $2}')
if [[ $SYSTEMD_VERSION -lt 246 ]]; then
    echo "Systemd version must be 246+ in order to enable consul dns."
    echo "Please update your chromeos container to Debian Bullseye by running"
    # shellcheck disable=SC2016
    echo '`sudo bash /opt/google/cros-containers/bin/upgrade_container DEBIAN_BULLSEYE`'
fi

# Since systemd-resolved listens on 127.0.0.53, we're going to also have it listen on the docker0 bridge.
# This will usually resolve to 172.17.0.1
DOCKER0_BRIDGE=$(docker network inspect bridge --format='{{(index .IPAM.Config 0).Gateway}}')

# Set up consul dns forwarding
sudo tee /etc/systemd/resolved.conf.d/consul.conf << EOF
# This forwards all requests to our extra stub listener to Consul DNS.
[Resolve]
DNS=127.0.0.1:8600
DNSSEC=false
Domains=~.
DNSStubListenerExtra=${DOCKER0_BRIDGE}
EOF

sudo systemctl enable systemd-resolved
sudo systemctl restart systemd-resolved
