#!/bin/bash

set -euo pipefail

# WARNING: This is not idempotent (yet). Only run this once

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
echo "DNSStubListenerExtra=$DOCKER0_BRIDGE" | sudo tee --append /etc/systemd/resolved.conf

sudo systemctl enable systemd-resolved
sudo systemctl restart systemd-resolved

#set up static dns manually
sudo tee /etc/systemd/resolved.conf.d/dns_servers.conf << EOF
[Resolve]
DNS=1.1.1.1 1.0.0.1 2606:4700:4700::1111 2606:4700:4700::1001
FallbackDNS=8.8.8.8 8.8.4.4 2001:4860:4860::8888 2001:4860:4860::8844
Domains=~.
EOF

# Set up consul dns forwarding
sudo tee /etc/systemd/resolved.conf.d/consul.conf << EOF
[Resolve]
DNS=127.0.0.1:8600
DNSSEC=false
Domains=~consul
EOF

# backup the old resolv.conf
sudo mv /etc/resolv.conf /etc/resolv.old.conf
# Switch resolv.conf to use systemd-resolve's stub
sudo ln -s /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf

sudo systemctl enable systemd-resolved
sudo systemctl restart systemd-resolved
