#!/usr/bin/env bash

set -euo pipefail

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
    org-management
    osquery-generator
    plugin-bootstrap
    plugin-registry
    plugin-work-queue
    provisioner
    # Heads up: Adding `rust-integration-tests` here? Reconsider!
    # It's 9GB and Cloudsmith space is pricy!
    # https://github.com/grapl-security/grapl/pull/1296
    sysmon-generator
)

cloudsmith_tag() {
    local -r service="${1}"
    local -r tag="${2}"
    echo "${CLOUDSMITH_DOCKER_REGISTRY}/${service}:${tag}"
}

echo "--- Building all ${TAG} images"
make build-for-push

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
        "${service}:${TAG}" \
        "${new_tag}"

    echo "--- :docker: Push ${new_tag}"
    docker push "${new_tag}"
done

########################################################################
# Determine whether or not this image is "new"
#
# Cloudsmith apparently has a bug that affects promotions when an
# artifact already exists in the destination repository. It seems to
# detect that the artifact is present and doesn't overwrite it, but it
# also doesn't carry tags / labels over. Thus, when we have services
# that don't change, we end up losing them in Cloudsmith.
#
# Until we can inspect the source of our Rust services to determine if
# they need a new image, we will build the images, but then query the
# upstream registry to see if that content exists there already or
# not.
#
# This does require us to build the images first (which wastes a bit
# of time). It also requires us to push the images to our `raw`
# repository first in order to obtain the image's sha256 checksum
# (there doesn't appear to be a way to do this purely locally,
# amazingly enough!). It also requires this script to be aware that
# we'll ultimately be promoting to our `testing` repository. These are
# all unfortunate, but it does allow us to sidestep this Cloudsmith
# bug. More importantly, it should make deployments quicker and more
# responsive, since services should churn less (they'll only restart
# when a new image is available, rather than for every single
# deployment). As such, we should keep this general logic even after
# the Cloudsmith bug is fixed.

# This is the list of services that actually have different images.
new_services=()

# This is where our images will ultimately be promoted to. It is the
# registry we'll need to query to see if an image with the same
# content already exists.
readonly UPSTREAM_REGISTRY="docker.cloudsmith.io/grapl/testing"

# It seems you can only get the sha256 sum of an image after pushing
# it to a registry. Fun.
#
# Returns a string like `sha256:deadbeef....`
sha256_of_image() {
    docker manifest inspect --verbose "${1}" | jq --raw-output '.Descriptor.digest'
}

# Returns 0 if it is present; 1 if not.
#
# We'll go ahead and allow the output to go to our logs; that will
# help debugging.
image_present_upstream() {
    docker manifest inspect "${1}"
}

echo "--- :cloudsmith::sleuth_or_spy: Checking upstream repository to determine what to promote"

for service in "${services[@]}"; do
    echo "--- :cloudsmith: Checking '${service}:${TAG}' in 'grapl/testing'"
    raw_repository_tag="$(cloudsmith_tag "${service}" "${TAG}")"
    sha256="$(sha256_of_image "${raw_repository_tag}")"
    echo "${raw_repository_tag} has identifier '${sha256}'"
    upstream_sha256_identifier="${UPSTREAM_REGISTRY}/${service}@${sha256}"

    echo "Checking the existence of '${upstream_sha256_identifier}'"
    if ! image_present_upstream "${upstream_sha256_identifier}"; then
        echo "Image not present upstream; adding '${service}' to the list of images to promote"
        new_services+=("${service}")
    else
        echo "Image already found upstream; nothing else to be done"
    fi
done

# Now that we've filtered out things that already exist upstream, we
# only need to care about the new stuff.
artifact_json "${TAG}" "${new_services[@]}" > "$(artifacts_file_for containers)"
