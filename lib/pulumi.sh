#!/usr/bin/env bash

# Pulumi Helper Functions

# All our Pulumi projects exist in our single organization.
readonly PULUMI_ORG="grapl"

# By convention, all of our Pulumi code exists in the `pulumi`
# directory in the repository root
readonly PULUMI_DIR="pulumi"

# When managing multiple projects and their stacks, we'll pass around
# qualified names in the form of `project/stack`. This is a shortened
# version of the fully-qualified name that Pulumi uses of
# `organization/project/stack`.
#
# Because of how we are currently organizing the code for our
# projects, they are stored in directories that are valid Python
# module names (specifically, they use underscores rather than
# hyphens). The formal project name in Pulumi, however, uses hyphens
# instead of underscores (mainly for cosmetic purposes, avoiding a
# mixture of hyphens and underscores in various generated names).
#
# We can account for this distinction with helper functions.

# Extract the project name from a "project/stack" pair.
#
#     split_project "foo/bar"
#     # => foo
#
split_project() {
    local -r input="${1}"
    cut --only-delimited --fields=1 --delimiter=/ <<< "${input}"
}

# Extract the stack name from a "project/stack" pair.
#
#     split_project "foo/bar"
#     # => bar
#
split_stack() {
    local -r input="${1}"
    cut --only-delimited --fields=2 --delimiter=/ <<< "${input}"
}

# Expands a project/stack name into the full, organization-qualified
# one.
#
# This will be used for the `--stack` value in various `pulumi` CLI
# invocations. Technically (since the project is already determined
# from the directory `pulumi` is invoked from), this can simply be of
# the form "<ORGANIZATION>/<STACK>", but
# "<ORGANIZATION>/<PROJECT>/<STACK>" is also accepted.
#
# So we don't have to come up with a distinction between the two- and
# three-part forms (and a three-part form is arguably "more fully"
# qualified anyway), we'll just conventionally use the three-part one.
#
#     fully_qualified_stack_name "foo/bar"
#     # => grapl/foo/bar
#
fully_qualified_stack_name() {
    local -r input="${1}"
    local -r project="$(split_project "${input}")"
    local -r stack="$(split_stack "${input}")"
    echo "${PULUMI_ORG}/${project}/${stack}"
}

# Returns the full path (from the repository root) of the directory
# for the given Pulumi project.
#
#     project_directory "foo-bar/testing"
#     # => pulumi/foo_bar
#
project_directory() {
    local -r input="${1}"
    local -r dir_name="$(split_project "${input}" | tr - _)"
    echo "${PULUMI_DIR}/${dir_name}"
}

# Expand a project/stack name into the full path (from the root of the
# repository) to its corresponding configuration file.
#
#     stack_file_path "foo-bar/testing"
#     # => pulumi/foo_bar/Pulumi.testing.yaml
#
stack_file_path() {
    local -r input="${1}"

    local -r project_dir="$(project_directory "${input}")"
    local -r stack="$(split_stack "${input}")"

    echo "${project_dir}/Pulumi.${stack}.yaml"
}
