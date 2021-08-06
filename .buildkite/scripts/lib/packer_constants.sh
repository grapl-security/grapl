#!/usr/bin/env bash
# shellcheck disable=SC2034

# This file contains various constants that are used in more than one
# place in our pipeline scripts.
#
# They should all be marked as `readonly`, be named in
# SCREAMING_SNAKE_CASE, and ordered alphabetically when possible.
########################################################################

# These are specified in `local.image_name`
readonly PACKER_IMAGE_NAME_SERVER="grapl-nomad-consul-server"
readonly PACKER_IMAGE_NAME_CLIENT="grapl-nomad-consul-client"
readonly PACKER_IMAGE_NAMES=(
    "${PACKER_IMAGE_NAME_SERVER}"
    "${PACKER_IMAGE_NAME_CLIENT}"
)

readonly PACKER_MANIFEST_SUFFIX=".packer-manifest.json"
