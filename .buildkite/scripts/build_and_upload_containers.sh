#!/usr/bin/env bash

set -euo pipefail

# TODO:
# - Build Containers for Fargate services
# - Upload them to Cloudsmith
# - Record them as artifacts for subsequent processing

# ISSUES:
# - How to do this selectively? I only really want to do this if we've
#   got real changes to build
# - Can this be decomposed in such a way that we can do it on a
#   per-container basis, rather than having to manage everything globally?

source .buildkite/scripts/lib/artifacts.sh
source .buildkite/scripts/lib/version.sh

# This variable is used in the docker-compose.build.yml file
TAG="$(timestamp_and_sha_version)"
export TAG

# TODO: This name may change
readonly CLOUDSMITH_DOCKER_REGISTRY="docker.cloudsmith.io/grapl/raw"

# These are defined in docker-compose.build.yml. There are other
# services defined in that file for other reasons; we do not need to
# build them all.
services=(
    analyzer-dispatcher
    analyzer-executor
    e2e-tests
    engagement-creator
    graph-merger
    graphql-endpoint
    grapl-web-ui
    model-plugin-deployer
    node-identifier
    node-identifier-retry
    osquery-generator
    provisioner
    sysmon-generator
)

cloudsmith_tag() {
    local -r service="${1}"
    local -r tag="${2}"
    echo "${CLOUDSMITH_DOCKER_REGISTRY}/${service}:${tag}"
}

echo "--- Building all ${TAG} images"
make build build-test-e2e

for service in "${services[@]}"; do
    # Re-tag the container we just built so we can upload it to
    # Cloudsmith.
    #
    # The other alternative is to embed this directly into the
    # docker-compose.build.yml file, but that is probably a bit
    # premature.
    new_tag="$(cloudsmith_tag "${service}" "${TAG}")"
    echo "--- :docker: Retagging ${service} container to ${new_tag}"
    docker tag \
        "grapl/${service}:${TAG}" \
        "${new_tag}"

    echo "--- :docker: Push ${new_tag}"
    docker push "${new_tag}"
done

# TODO: Do we want to put the complete container image name in for
# each service? Perhaps not, particularly if we're going to be
# promoting containers across repositories
artifact_json "${TAG}" "${services[@]}" > "$(artifacts_file_for containers)"
