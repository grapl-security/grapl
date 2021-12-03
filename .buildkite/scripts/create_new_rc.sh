#!/usr/bin/env bash

set -euo pipefail

source .buildkite/scripts/lib/artifacts.sh
source .buildkite/shared/lib/rc.sh

# TODO: if we passed in the stacks, or otherwise computed them, then
# this would be a fully general script.

# This is the list of all the project/stacks we have in this
# repository.
readonly -a STACKS=(
    grapl/testing
    # FYI: this will reference the `grapl/integration-tests/testing` stack
    integration-tests/testing
)

# TODO: Add this to environment file
export GIT_AUTHOR_NAME="Grapl CI/CD"
export GIT_AUTHOR_EMAIL="grapl-cicd@graplsecurity.com"

# Download artifacts file
echo -e "--- :buildkite: Download artifacts file"
if (buildkite-agent artifact download "${ALL_ARTIFACTS_JSON_FILE}" .); then
    artifacts_json="$(cat "${ALL_ARTIFACTS_JSON_FILE}")"
else
    echo "^^^ +++" # Un-collapses this section in Buildkite, making it more obvious we couldn't download
    artifacts_json="{}"
fi

# TODO: If we have no new artifacts, and if no Pulumi code has changed,
# then we don't really need to create a release candidate, right?
#
# We could use Pants to determine this, since all the Pulumi code is
# (theoretically) addressable by Pants.

# Create new release candidate.
echo -e "--- :github: Creating new release candidate"
create_rc "${artifacts_json}" "${STACKS[@]}"
