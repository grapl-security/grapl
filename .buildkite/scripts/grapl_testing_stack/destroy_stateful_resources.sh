#!/usr/bin/env bash

################################################################################
# https://github.com/grapl-security/issue-tracker/issues/793
# This script is a stop-gap measure while we await organization IDs.
# It will destroy dgraph and dynamodb state in an AWS Grapl sandbox.
# (You'll likely want to `pulumi update` immediately afterwards!)
################################################################################

set -euo pipefail

########################################################################
# This logic is mostly stolen from pulumi-buildkite-plugin, and sets up
# a `pulumi` on PATH.
if [[ -z "${SKIP_VENV:-}" ]]; then
    echo -e "--- :python: Installing dependencies"
    build-support/manage_virtualenv.sh populate
    export VIRTUAL_ENV=build-support/venv
    PATH="$(pwd)/${VIRTUAL_ENV}/bin":${PATH}
    export PATH
fi
########################################################################

project_dir="pulumi/grapl"
stack="${1}"

echo -e "--- :pulumi: Log in"
pulumi login

# The default for this is `true`, which was set to reduce unnecessary
# network calls, but is actually useful to have in AWS! This lets us
# get credentials from the IMDS.
#
# While it *technically* means that our configuration is not precisely
# what is in version control, it's a configuration that doesn't
# actually have any impact on the infrastructure being created.
#
# For further background, see:
# https://github.com/pulumi/pulumi-aws/pull/1288
# https://github.com/pulumi/pulumi-aws/issues/1636
pulumi config set \
    aws:skipMetadataApiCheck false \
    --cwd="${project_dir}" \
    --stack="${stack}"

# turn '["a", "b"]' into bash-array (--target=a --target=b)
urns=$(pulumi stack output stateful-resource-urns --stack="${stack}")
mapfile -t target_args < <(echo "${urns}" | jq -r '. | map("--target=" + .) | .[]')

echo -e "--- :pulumi: Destroying stateful resources for ${stack}"

# Attempt to destroy each target, one at a time.
# This can be de-for-loop-ified after
# https://github.com/pulumi/pulumi/issues/3304
for target_arg in "${target_args[@]}"; do
    pulumi destroy \
        --cwd="${project_dir}" \
        --stack="${stack}" \
        --show-replacement-steps \
        --non-interactive \
        --diff \
        --yes \
        --refresh \
        --message="Destroying stateful resources" \
        --target-dependents \
        "${target_arg}" ||
        true
    # ^ Allow errors; each destroy is best-effort and should allow for
    # failed destroys of targets that no longer exist.
done
