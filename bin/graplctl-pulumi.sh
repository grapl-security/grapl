#!/usr/bin/env bash

# Call `graplctl` with awareness of the outputs of a particular
# Pulumi stack.
########################################################################
#
# This script is intended to be a drop-in replacement for any call to
# `graplctl` directly. All options and arguments passed to this script
# will be passed to `graplctl` without any changes.
#
# In order to specify the Pulumi stack to operate on, you must set
# `GRAPLCTL_PULUMI_STACK` in your environment. The value must be the
# name of a Pulumi stack from the `grapl` project (i.e., from
# `pulumi/grapl` in this repository).
#
# All the outputs of this stack will be injected into the environment
# that `graplctl` will ultimately be called in. In this way, users
# will not need to manually compose long invocations with many
# required arguments, nor will they have to manually juggle
# environment variables as they switch among multiple Pulumi stacks,
# or among different incarnations of the same stack.
#
# Note that this script does not magically take care of all arguments
# that `graplctl` takes, nor does it handle the values of all
# environment variables to which `graplctl` responds. It *only*
# injects Pulumi stack outputs into the environment, and then calls
# `graplctl` in that environment. Any values that do not come from a
# Pulumi stack are still the responsibility of the user.
#
# Stack output names are converted to SCREAMING_SNAKE_CASE, and
# prepended with "GRAPL_" before being injected into the
# environment. Thus, a stack output of `dgraph-config-bucket` becomes
# `GRAPL_DGRAPH_CONFIG_BUCKET`. These environment variable names will
# correspond to an argument of a `graplctl` command.
#
# Currently, no special handling is given to secret outputs, or
# outputs with non-scalar values, simply because `graplctl` has no
# need to access such values. This wrapper script is anticipated to be
# needed for a relatively short amount of time, so this "shortcut"
# should be fine.
#
# It should be noted that this wrapper script does not have to be
# used; it is merely a convenience. Users may continue to specify all
# argument values for graplctl explicitly, and call that binary
# directly.
#
# This script assumes that a `graplctl` binary is present in the `bin`
# directory at the root of the repository (as you would get from
# running `make graplctl`). This script also assumes that it is being
# called from within the grapl repository (though not necessarily
# directly from the root of the repository).
#
# Invocation Example
########################################################################
#
#     export GRAPLCTL_PULUMI_STACK=grapl/my-sandbox
#     graplctl-pulumi.sh dgraph create --instance-type=i3.large
#     # etc.

set -euo pipefail

########################################################################
# Helper Functions
########################################################################

# Dump Pulumi stack outputs from the `grapl` project as JSON.
#
# DOES NOT currently decode any secrets!
#
# Assumes it is being invoked from the root of the grapl repository.
stack_outputs() {
    local -r stack="${1}"
    pulumi stack output \
        --cwd="pulumi/grapl" \
        --json \
        --stack="${stack}"
}

# Converts Pulumi stack outputs into environment variables.
#
# Hyphens are converted to underscores, "GRAPL_" is prepended, and
# everything is uppercased.
#
# Implicitly assumes outputs are a flat object; that is, that all
# values are scalars.
#
# Output is `<ENV_VAR>=<VALUE>`, one per line, e.g.:
#
#     GRAPL_FOO=one
#     GRAPL_BAR=two
#     GRAPL_BAZ=three
#
grapl_envvars_from_outputs() {
    local -r json="${1}"
    jq --raw-output \
        'to_entries | .[] |
       "GRAPL_" + (.key | gsub("-";"_") | ascii_upcase) + "=" + .value' \
        <<< "${json}"
}

########################################################################
# Main Script Logic
########################################################################

# Requires this environment variable to have already been set.
readonly pulumi_stack="${GRAPLCTL_PULUMI_STACK}"

(
    # Perform these operations from the root of the repository, to
    # make locating the Pulumi data and the `graplctl` binary more
    # straightforward.
    REPOSITORY_ROOT="$(git rev-parse --show-toplevel)"
    readonly REPOSITORY_ROOT
    cd "${REPOSITORY_ROOT}"

    # Collect the outputs of the given stack
    stack_json="$(stack_outputs "${pulumi_stack}")"

    # Export all the stack outputs into the environment of this script
    set -o allexport
    # shellcheck disable=SC1090
    source <(grapl_envvars_from_outputs "${stack_json}")
    set +o allexport

    # Invoke graplctl, passing along all the arguments unchanged
    ./bin/graplctl "${@}"
)
