#!/usr/bin/env bash

set -euo pipefail

# Should be just the stack name, NOT including the organization (e.g.,
# "testing", not "grapl/testing")
readonly stack_name="${1}"

# This is used by the graplctl-pulumi.sh script to hook into the
# Pulumi stack outputs.
export GRAPLCTL_PULUMI_STACK="grapl/${stack_name}"

# These are currently required by graplctl
export DEPLOYMENT_NAME="${stack_name}"
GRAPL_REGION="$(pulumi config get aws:region --stack="${GRAPLCTL_PULUMI_STACK}" --cwd=pulumi/grapl)"
export GRAPL_REGION
export GRAPL_VERSION="${BUILDKITE_PIPELINE_ID}"

########################################################################

echo "--- :building_construction: Building graplctl binary"
make graplctl

echo "--- :broom: Clean up previous deployments"
./bin/graplctl-pulumi.sh aws wipe-state --yes || true
./bin/graplctl-pulumi.sh dgraph destroy --yes || true

echo "--- :building_construction: Create Dgraph cluster"
./bin/graplctl-pulumi.sh dgraph create --instance-type=i3.large

echo "--- :house_with_garden: Provision environment"
./bin/graplctl-pulumi.sh aws provision --yes

echo "--- :arrow_up::cloud: Uploading analyzers"
./bin/graplctl-pulumi.sh upload analyzer \
    --analyzer_main_py etc/local_grapl/unique_cmd_parent/main.py

./bin/graplctl-pulumi.sh upload analyzer \
    --analyzer_main_py etc/local_grapl/suspicious_svchost/main.py

echo "--- :arrow_up::cloud: Uploading sample data"
./bin/graplctl-pulumi.sh upload sysmon \
    --logfile etc/sample_data/eventlog.xml

echo "--- :sob: Sleep a little while for Reasons (TM)"
for i in {1..60}; do
    echo "Sleeping ${i}/60 seconds"
    sleep 1
done

echo "--- :running::running::running: Running tests"
./bin/graplctl-pulumi.sh aws test
