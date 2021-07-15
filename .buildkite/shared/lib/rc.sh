#!/usr/bin/env bash

set -euo pipefail

# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/json_tools.sh"
# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/util.sh"
# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/pulumi.sh"

# Given a project and stack name, and a flat JSON object as an input
# string, adds each key-value pair the Pulumi configuration for that
# stack.
#
# Input of "cicd", "production", '{"foo":"123","bar":"456"}' would end up running:
#
#     pulumi config set --path "artifacts.foo" "123" --cwd=pulumi/cicd --stack=grapl/production
#     pulumi config set --path "artifacts.bar" "456" --cwd=pulumi/cicd --stack=grapl/production
#

add_artifacts() {
    local -r stack="${1}"
    local -r input_json="${2}"

    flattened_input_json=$(flatten_json "${input_json}")

    jq -r 'to_entries | .[] | [.key, .value] | @tsv' <<< "${flattened_input_json}" |
        while IFS=$'\t' read -r key value; do
            pulumi config set \
                --path "artifacts.${key}" \
                "${value}" \
                --cwd="$(project_directory "${stack}")" \
                --stack="$(fully_qualified_stack_name "${stack}")"
        done
}

# Generate a commit message for this containing metadata about the
# artifacts that were updated, if any.
commit_message() {
    local -r input_json="${1}"

    if had_new_artifacts "${input_json}"; then
        echo "Create new release candidate with updated deployment artifacts"
        echo
        echo "Updated the following artifact versions:"
        echo
        jq -r '
        to_entries | .[] |
        "- " + .key + " => " + .value
        ' <<< "${input_json}"
    else
        echo "Create new release candidate"
    fi
    echo
    echo "Generated from ${BUILDKITE_BUILD_URL}"
}

had_new_artifacts() {
    local -r input_json="${1}"
    num_artifacts=$(jq 'length' <<< "${input_json}")
    if (("${num_artifacts}" == 0)); then
        return 1
    fi
    return 0
}

# Returns a JSON object for the `artifacts` configuration key in the
# given project and stack, or `{}` if the key is not present.
#
# By convention, we store all our pinned artifact versions in Pulumi
# config in this manner.
existing_artifacts() {
    local -r stack="${1}"

    pulumi config get artifacts \
        --cwd="$(project_directory "${stack}")" \
        --stack="$(fully_qualified_stack_name "${stack}")" ||
        echo "{}"
}

# Given a short stack name and a flat JSON object of artifact-version
# pairs:
#
# - merges any configuration changes for the stack from `main` into
#   `rc`
# - preserves any artifact versions that were previously specified
#   on `rc`
# - Adds the new artifacts from this pipeline run to the stack
#   configuration
# - Adds the updated stack configuration to the git staging area
#
# Assumes that we are currently on the `rc` branch, and are always
# pulling core config updates from the `main` branch.
#
update_stack_config_for_commit() {
    local -r project_stack="${1}"
    local -r new_artifacts="${2}"

    local -r stack_file="$(stack_file_path "${project_stack}")"

    # First, we want to preserve any artifact versions that are
    #already in the `rc` branch.
    echo -e "--- Extracting pinned artifact versions from rc branch"
    existing_rc_artifacts="$(existing_artifacts "${project_stack}")"
    jq '.' <<< "${existing_rc_artifacts}"

    # Now that we've captured the artifact versions from this version
    # of the config file, we'll copy back the original contents of the
    # config from the `main` branch.
    #
    # The idea is that if we add new, non-artifact configuration during
    # the course of normal development, we want to carry that over to the
    # `rc` branch.
    echo -e "--- Restoring config file from main branch"
    git show "main:${stack_file}" > "${stack_file}"
    cat "${stack_file}"

    # Now that we have our base configuration reestablished, we need
    # to add back the artifact versions that were on the `rc` branch
    # already.
    echo -e "--- Adding pinned artifacts back to config file"
    add_artifacts "${project_stack}" "${existing_rc_artifacts}"
    cat "${stack_file}"

    # Finally, we can layer on any new or updated artifact versions that
    # were generated in *this build*. This line is the point of this
    # entire script.
    echo -e "--- Adding new artifact pins to config file"
    add_artifacts "${project_stack}" "${new_artifacts}"
    cat "${stack_file}"

    # Add the updated configuration file to our already-in-progress merge
    # commit.
    echo -e "--- :git: Adding config file to in-progress merge commit"
    git add --verbose "${stack_file}"
}

create_rc() {
    # This fundamentally assumes that we're running on the main branch!

    # A JSON object string
    local -r new_artifacts="${1}"

    # All other arguments are all the "project/stack" config files to
    # update.
    shift
    local -ra stacks=("${@}")

    # We have to log in before we can update any configuration values.
    echo -e "--- :pulumi: Logging in to Pulumi"
    pulumi login

    echo -e "--- :git: Checking out the rc branch"
    git checkout rc

    echo -e "--- :git: Begin merge of main branch to rc"
    # TODO: For some as-yet unknown reason, it appears that we MUST
    # set the author and email in a config file for it to take
    # effect. Simply having the values in the environment doesn't
    # work, nor does specifying a value at commit-time with
    # `--author`.
    git config user.name "${GIT_AUTHOR_NAME}"
    git config user.email "${GIT_AUTHOR_EMAIL}"

    # We use the recursive/theirs strategy here to preserve the
    # conflicts from the rc branch preferentially (this should only
    # involve the Pulumi stack config files, which is exactly what we
    # want). As we process the files further, we'll resolve any
    # semantic changes we truly wish to preserve.
    git merge \
        --no-ff \
        --no-commit \
        --strategy=recursive \
        --strategy-option=ours \
        main

    for stack in "${stacks[@]}"; do
        update_stack_config_for_commit "${stack}" "${new_artifacts}"
    done

    # Finalize the commit, with a helpful, metadata-laden commit message.
    echo -e "--- :git: Finalizing commit"
    git commit \
        --message="$(commit_message "${new_artifacts}")"
    git --no-pager show

    # Finally, push it up to Github!
    if is_real_run; then
        echo -e "--- :github: Pushing rc branch to Github"
        git push --verbose
    else
        echo -e "--- :no_good: Would have pushed rc branch to Github"
    fi
}
