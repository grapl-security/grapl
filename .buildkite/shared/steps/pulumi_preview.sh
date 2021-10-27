#!/usr/bin/env bash

# Run a `pulumi preview` on a given Pulumi stack.
#
# Assumptions:
# - Python virtualenv is managed by Pants via `build-support/manage_virtualenv.sh`
# - All Pulumi projects are stored in `pulumi/<project_name>`
# - Projects are named in kebab-case, but directories for the projects
#   are snake_case

set -euo pipefail

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/../lib/pulumi.sh"

# Given as "org/project/stack"
readonly project_stack="${1}"

echo -e "--- :python: Installing dependencies"
build-support/manage_virtualenv.sh populate

# shellcheck disable=SC1091
source build-support/venv/bin/activate

echo -e "--- :pulumi: Log in"
pulumi login

echo -e "--- :pulumi: Previewing changes to ${project_stack} infrastructure"
pulumi preview \
    --cwd="$(project_directory "${project_stack}")" \
    --stack="${project_stack}" \
    --show-replacement-steps \
    --non-interactive \
    --diff \
    --message="Previewing from ${BUILDKITE_BUILD_URL}"
