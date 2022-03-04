#!/usr/bin/env bash

# Helper script to run various commands inside a Docker "build
# environment" container.
#
# Probably shouldn't run this directly, in favor of interacting with
# it via the Makefile.

set -euo pipefail

# ANSI Formatting Codes
########################################################################
# Because who wants to remember all those fiddly details?

# CSI = "Control Sequence Introducer"
CSI="\e["
END=m

NORMAL=0
BOLD=1

RED=31
WHITE=37

RESET="${CSI}${NORMAL}${END}"

function _bold_color() {
    color="${1}"
    shift
    echo "${CSI}${BOLD};${color}${END}${*}${RESET}"
}

function bright_white() {
    _bold_color "${WHITE}" "${@}"
}

function bright_red() {
    _bold_color "${RED}" "${@}"
}

# Logging
########################################################################
# NOTE: All logs get sent to standard error

function log() {
    echo -e "${@}" >&2
}

# Handy for logging the exact command to be run, and then running it
function log_and_run() {
    log ❯❯ "$(bright_white "$(printf "%q " "${@}")")"
    "$@"
}

# Helper function to log information about what volumes we are
# mounting, and where.
function log_volume() {
    volume=${1}
    dest=${2}
    var=${3}
    log "$(bright_white "Mounting volume '${volume}' at ${dest}") \($(bright_red "\${${var}}")\)"
}

########################################################################

# Inspect an image to find the value of an environment variable set in
# it. Fails if the variable is not set.
#
# We do this to intelligently re-use information from the image,
# rather than having to keep it in sync between this script and the
# Dockerfile.
function env_from_image() {
    local -r image="${1}"
    local -r envvar="${2}"

    # Environment variables are presented as an array of strings of
    # the form "FOO=bar". We use `jq` to select the variable we're
    # looking for, and then `cut` to isolate the value.
    docker inspect "${image}" \
        --format='{{ json .Config.Env }}' |
        jq --raw-output --exit-status "map(select( . | contains(\"${envvar}\"))) | .[]" |
        cut --delimiter="=" --fields=2
}

########################################################################

# Shellcheck seems to think that `IMAGE` is confused with `image` from
# `env_from_image`; weird.
# shellcheck disable=SC2153
log "$(bright_white "Using build image '${IMAGE}'")"

target_dir="$(env_from_image "${IMAGE}" CARGO_TARGET_DIR)"
readonly target_dir
readonly target_volume="grapl-target"
log_volume ${target_volume} "${target_dir}" CARGO_TARGET_DIR

cargo_cache_dir="$(env_from_image "${IMAGE}" CARGO_HOME)"
readonly cargo_cache_dir
readonly cargo_volume="grapl-cargo-cache"
log_volume ${cargo_volume} "${cargo_cache_dir}" CARGO_HOME

rustup_dir="$(env_from_image "${IMAGE}" RUSTUP_HOME)"
readonly rustup_dir
readonly rustup_volume="grapl-rustup"
log_volume ${rustup_volume} "${rustup_dir}" RUSTUP_HOME

# All invocations of this build container will use these basic
# arguments.
common_args=(
    --rm
    # Note: `--interactive --tty` must be present if you want to have any
    # hope of cancelling the job using ^C (the alternative is to use
    # `docker kill`). Unfortunately, that also means that this won't be
    # able to be reliably run under `make` using the `-j`/`--jobs` flag.
    --interactive --tty
    # Our Rust source is mounted read/write because updates may need
    # to be made to Cargo.lock (e.g., when a new dependency is added
    # to a Cargo.toml file).
    --mount="type=bind,source=${REPO_ROOT}/src/rust,target=/grapl/rust"
    # There's no reason to make changes to the protobuf files in any
    # commands we'll run in this container, however, so they are
    # mounted read-only.
    --mount="type=bind,source=${REPO_ROOT}/src/proto,target=/grapl/proto,readonly"
    --mount="type=volume,source=${cargo_volume},target=${cargo_cache_dir}"
    --mount="type=volume,source=${rustup_volume},target=${rustup_dir}"
    # This allows us to be able to write files to mounted directories
    # with the same permissions as our workstation user.
    --user="$(id --user):$(id --group)"
    # We'll be dropped into our Rust source directory
    --workdir=/grapl/rust
)

# Most invocations will also mount the Cargo target volume.
normal_args=(
    --mount="type=volume,source=${target_volume},target=${target_dir}"
)

# Running Tarpaulin will *not* use the Cargo target volume because it
# will taint the target directory for subsequent runs, making them
# take a needlessly long time.
tarpaulin_args=(
    # We mount the `dist` directory from the repository root so we
    # have a place to dump our coverage statistics alongside the
    # statistics from our other languages. As such, it is mounted
    # read/write.
    --mount="type=bind,source=${REPO_ROOT}/dist,target=/dist"
    # We must set our secure computing profile to "unconfined" to
    # allow Tarpaulin to run properly by disabling ASLR.
    #
    # Technically, we could tweak the seccomp profile on on our
    # machine to get the same effect, but bypassing it altogether
    # works for our purposes today. See
    # https://github.com/xd009642/tarpaulin/issues/146#issuecomment-554492608
    # for some helpful background on this.
    --security-opt=seccomp=unconfined
)

# Based on the command we've been given to run, set up the proper
# `docker` invocation.
if [ "${2}" = "tarpaulin" ]; then
    # e.g., calling `cargo tarpaulin`
    log_and_run docker run \
        "${common_args[@]}" \
        "${tarpaulin_args[@]}" \
        -- \
        "${IMAGE}" \
        "${@}"
else
    # It's a "normal" invocation (`cargo build`, `cargo test`, etc.)
    log_and_run docker run \
        "${common_args[@]}" \
        "${normal_args[@]}" \
        -- \
        "${IMAGE}" \
        "${@}"
fi
