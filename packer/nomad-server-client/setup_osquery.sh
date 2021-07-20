#!/bin/sh

# Adapted from "Alternative Install Options -> RPM Linux" at
# https://osquery.io/downloads/official/4.9.0

set -euo pipefail

# Add repo
curl -L https://pkg.osquery.io/rpm/GPG | sudo tee /etc/pki/rpm-gpg/RPM-GPG-KEY-osquery
sudo yum-config-manager --add-repo https://pkg.osquery.io/rpm/osquery-s3-rpm.repo
sudo yum-config-manager --enable osquery-s3-rpm-repo

sudo yum install --assumeyes osquery

# Ensure it's all working correctly.
osqueryi --version
# Yes, the `i` at the end is not a typo