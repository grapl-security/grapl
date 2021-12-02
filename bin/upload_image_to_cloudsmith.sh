#!/usr/bin/env bash

########################################################################
# Example Usage
#
# bin/upload_image_to_cloudsmith.sh e2e-tests:dev
########################################################################

set -euo pipefail

DOCKER_PACKAGE_FILE=/tmp/upload_this_to_cloudsmith.docker
readonly DOCKER_PACKAGE_FILE

IMAGE_WITH_TAG="${1}"
readonly IMAGE_WITH_TAG

CLOUDSMITH_DOCKER_REPO_NAME="grapl/${USER}"
readonly CLOUDSMITH_DOCKER_REPO_NAME

########################################################################
# Main Script Logic
########################################################################

docker save -o "${DOCKER_PACKAGE_FILE}" "${IMAGE_WITH_TAG}"

cloudsmith push docker "${CLOUDSMITH_DOCKER_REPO_NAME}" "${DOCKER_PACKAGE_FILE}"

rm "${DOCKER_PACKAGE_FILE}"

echo "Your image is available at 'docker.cloudsmith.io/${CLOUDSMITH_DOCKER_REPO_NAME}/${IMAGE_WITH_TAG}'"
