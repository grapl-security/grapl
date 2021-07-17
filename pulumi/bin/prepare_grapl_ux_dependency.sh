#!/usr/bin/env bash

# Since Pulumi does not appear to have a native way to extract a
# tarball into an S3 bucket, we need to have a bit of external logic
# to download the artifact for our UX assets to the local machine and
# extract it into a well-known directory (here,
# /dist/grapl-ux-<VERSION>) that is accessible to our Pulumi code.
#
# If the Pulumi stack being deployed has a pinned version for the
# grapl-ux artifact, this script should be run prior to `pulumi up`.

set -euo pipefail

# A Pulumi stack name, such as "grapl/testing".
#
# This should correspond to a stack from our `pulumi/grapl` project.
readonly stack="${1}"

# Download a tar.gz artifact from Cloudsmith to `/tmp`, then extract
# it into the `dist` directory.
#
#    retrieve_targz_from_cloudsmith foo 0.0.1
#    # => Downloads foo.tar.gz, version 0.0.1 from our Cloudsmith
#    #    "raw" repository
#
retrieve_targz_from_cloudsmith() {
    # e.g. foo, not foo.tar.gz
    local -r base_filename="${1}"

    # This is the name of the "package" in the repository
    local -r file="${base_filename}.tar.gz"

    # This is the version of the package in the repository
    local -r version="${2}"

    # This is the name of the repository, not the artifact type
    # (though the artifact type is also "raw"). This may change in the
    # future as we mature in our repository usage patterns.
    local -r repository="raw"

    # We'll save the downloaded artifact in /tmp
    local -r savename="/tmp/${base_filename}-${version}.tar.gz"

    curl --output "${savename}" \
        "https://dl.cloudsmith.io/public/grapl/${repository}/raw/versions/${version}/${file}"

    # Extract the artifact into our dist directory. The directory has
    # the version appended to it to ensure that we're using the
    # correct assets in Pulumi.
    mkdir -p "dist/${base_filename}-${version}"
    tar --extract \
        --gunzip \
        --verbose \
        --directory="dist/${base_filename}-${version}" \
        --file="${savename}"
}

########################################################################
# Main Logic
########################################################################

(
    # NOTE: For simplicity and flexibility, all the functions,
    # directory references, etc., in this script are written assuming
    # they are being executed from the root of this repository.
    #
    # However, by running the main logic in a subshell and changing to
    # the root as our first operation, we can allow the script to be
    # run from any subdirectory.
    REPOSITORY_ROOT=$(git rev-parse --show-toplevel)
    readonly REPOSITORY_ROOT
    cd "${REPOSITORY_ROOT}"

    ux_version=$(
        cd pulumi/grapl
        pulumi config get --path artifacts.grapl-ux --stack="${stack}"
    ) || true
    readonly ux_version

    if [ -n "${ux_version}" ]; then
        # We have a version; retrieve and unpack
        retrieve_targz_from_cloudsmith grapl-ux "${ux_version}"
    else
        # We don't have a version; use local build
        make build-ux
    fi
)
