#!/usr/bin/env bash

# mock `pulumi` binary
#
# This prevents the actual `pulumi` binary from being executed during
# these tests. Every invocation is logged to a file.
#
# To make assertions, simply inspect the contents of the file to
# ensure that the expected commands _would_ have been invoked.
#
# In the case of the `pulumi config get artifacts`, we actually do
# need to simulate a functioning binary. In that case, the behavior is
# governed by the $EXISTING_ARTIFACTS environment variable, which may
# be set in a test. If it is not present, the command errors out, as
# it would in "real life". If it is present, its value is returned.
pulumi() {
    echo "${FUNCNAME[0]} $*" >> "${ALL_COMMANDS}"

    case "$*" in
        config\ get\ artifacts*)
            # Expects EXISTING_ARTIFACTS to be a valid, non-empty JSON object
            if [ -n "${EXISTING_ARTIFACTS:-}" ]; then
                echo "${EXISTING_ARTIFACTS}"
            else
                # Simulates the error condition where the expected
                # config key was not present
                return 255
            fi
            ;;
        *) ;;
    esac
}

# Simple git binary mock that just records its invocations
git() {
    echo "${FUNCNAME[0]} $*" >> "${ALL_COMMANDS}"
}

recorded_commands() {
    if [ -f "${ALL_COMMANDS}" ]; then
        cat "${ALL_COMMANDS}"
    fi
}

oneTimeSetUp() {
    export BUILDKITE_BUILD_URL="https://buildkite.com/grapl/pipeline-infrastructure-verify/builds/2112"
    export ALL_COMMANDS="${SHUNIT_TMPDIR}/all_commands"
    export GIT_AUTHOR_NAME="Testy McTestface"
    export GIT_AUTHOR_EMAIL="tests@tests.com"

    # shellcheck source-path=SCRIPTDIR
    source "$(dirname "${BASH_SOURCE[0]}")/rc.sh"
}

setUp() {
    # Ensure any recorded commands from the last test are removed so
    # we start with a clean slate.
    rm -f "${ALL_COMMANDS}"

    # Some functions under test assume they are running from the
    # repository root, and will have a `pulumi` directory present,
    # along with individual project directories within. We run these
    # tests through Pants, which copies the test files into an
    # isolated temporary directory for execution. Importantly, this
    # directory does not have any of these subdirectories.
    #
    # To enable those tests to run, well just create them. At this
    # time, we don't actually need any of the files.
    mkdir -p pulumi/cicd
}

tearDown() {
    rm -Rf pulumi
}

test_add_artifacts_with_artifacts() {
    add_artifacts cicd/production '{"app1":"v1.0.0","app2":"v1.2.0","nested_map":{"k":"v"}}'

    expected=$(
        cat << EOF
pulumi config set --path artifacts.app1 v1.0.0 --cwd=pulumi/cicd --stack=grapl/cicd/production
pulumi config set --path artifacts.app2 v1.2.0 --cwd=pulumi/cicd --stack=grapl/cicd/production
pulumi config set --path artifacts.nested_map.k v --cwd=pulumi/cicd --stack=grapl/cicd/production
EOF
    )

    assertEquals "The expected pulumi commands were not run" \
        "${expected}" \
        "$(recorded_commands)"
}

test_add_artifacts_without_artifacts() {
    add_artifacts cicd/production '{}'
    assertNull "Should not have run any pulumi commands" \
        "$(recorded_commands)"
}

# Ensure we generate a commit message with information about the
# artifacts generated during this pipeline run.
test_commit_message_with_artifacts() {
    actual="$(commit_message '{"app1":"v1.0.0","app2":"v2.0.0"}')"
    expected=$(
        cat << EOF
Create new release candidate with updated deployment artifacts

Updated the following artifact versions:

- app1 => v1.0.0
- app2 => v2.0.0

Generated from https://buildkite.com/grapl/pipeline-infrastructure-verify/builds/2112
EOF
    )
    assertEquals "${expected}" "${actual}"
}

# Ensure we generate a sane commit message even if we don't generate
# any new artifacts during this pipeline run.
test_commit_message_without_artifacts() {
    actual="$(commit_message '{}')"
    expected=$(
        cat << EOF
Create new release candidate

Generated from https://buildkite.com/grapl/pipeline-infrastructure-verify/builds/2112
EOF
    )
    assertEquals "${expected}" "${actual}"
}

test_had_new_artifacts() {
    input='{"app1":"v6.6.6"}'
    assertTrue "A JSON object with keys should 'have artifacts'" \
        "had_new_artifacts ${input}"

    input='{}'
    assertFalse "An empty JSON object has no artifacts" \
        "had_new_artifacts ${input}"
}

test_existing_artifacts_with_artifacts() {
    actual="$(EXISTING_ARTIFACTS='{"app1":"v1.0.0"}' existing_artifacts cicd/production)"
    assertEquals '{"app1":"v1.0.0"}' "${actual}"
    assertEquals \
        "pulumi config get artifacts --cwd=pulumi/cicd --stack=grapl/cicd/production" \
        "$(recorded_commands)"
}

test_existing_artifacts_without_artifacts() {
    actual="$(existing_artifacts cicd/production)"
    assertEquals "{}" "${actual}"
    assertEquals \
        "pulumi config get artifacts --cwd=pulumi/cicd --stack=grapl/cicd/production" \
        "$(recorded_commands)"
}

test_update_stack_config_for_commit_with_new_artifacts_without_existing() {
    update_stack_config_for_commit "cicd/production" '{"app1":"v9.9.9","app2":"v1.0alpha"}'

    expected=$(
        cat << EOF
pulumi config get artifacts --cwd=pulumi/cicd --stack=grapl/cicd/production
git show main:pulumi/cicd/Pulumi.production.yaml
pulumi config set --path artifacts.app1 v9.9.9 --cwd=pulumi/cicd --stack=grapl/cicd/production
pulumi config set --path artifacts.app2 v1.0alpha --cwd=pulumi/cicd --stack=grapl/cicd/production
git add --verbose pulumi/cicd/Pulumi.production.yaml
EOF
    )

    assertEquals "${expected}" "$(recorded_commands)"
}

test_update_stack_config_for_commit_without_new_artifacts_without_existing() {
    update_stack_config_for_commit "cicd/production" '{}'

    expected=$(
        cat << EOF
pulumi config get artifacts --cwd=pulumi/cicd --stack=grapl/cicd/production
git show main:pulumi/cicd/Pulumi.production.yaml
git add --verbose pulumi/cicd/Pulumi.production.yaml
EOF
    )

    assertEquals "${expected}" "$(recorded_commands)"
}

test_update_stack_config_for_commit_with_new_artifacts_with_existing() {
    (
        EXISTING_ARTIFACTS='{"app1":"v9.9.8","app3":"0.0.1"}'
        update_stack_config_for_commit "cicd/production" '{"app1":"v9.9.9","app2":"v1.0alpha"}'
    )

    expected=$(
        cat << EOF
pulumi config get artifacts --cwd=pulumi/cicd --stack=grapl/cicd/production
git show main:pulumi/cicd/Pulumi.production.yaml
pulumi config set --path artifacts.app1 v9.9.8 --cwd=pulumi/cicd --stack=grapl/cicd/production
pulumi config set --path artifacts.app3 0.0.1 --cwd=pulumi/cicd --stack=grapl/cicd/production
pulumi config set --path artifacts.app1 v9.9.9 --cwd=pulumi/cicd --stack=grapl/cicd/production
pulumi config set --path artifacts.app2 v1.0alpha --cwd=pulumi/cicd --stack=grapl/cicd/production
git add --verbose pulumi/cicd/Pulumi.production.yaml
EOF
    )

    assertEquals "${expected}" "$(recorded_commands)"
}

test_update_stack_config_for_commit_without_new_artifacts_with_existing() {
    (
        EXISTING_ARTIFACTS='{"app1":"v9.9.8","app3":"0.0.1"}'
        update_stack_config_for_commit "cicd/production" '{}'
    )

    expected=$(
        cat << EOF
pulumi config get artifacts --cwd=pulumi/cicd --stack=grapl/cicd/production
git show main:pulumi/cicd/Pulumi.production.yaml
pulumi config set --path artifacts.app1 v9.9.8 --cwd=pulumi/cicd --stack=grapl/cicd/production
pulumi config set --path artifacts.app3 0.0.1 --cwd=pulumi/cicd --stack=grapl/cicd/production
git add --verbose pulumi/cicd/Pulumi.production.yaml
EOF
    )

    assertEquals "${expected}" "$(recorded_commands)"
}

test_create_rc_webhook() {
    (
        BUILDKITE_SOURCE=webhook
        create_rc "{}" cicd/production cicd/testing
    )

    expected=$(
        cat << EOF
pulumi login
git checkout rc
git config user.name Testy McTestface
git config user.email tests@tests.com
git merge --no-ff --no-commit --strategy=recursive --strategy-option=ours main
pulumi config get artifacts --cwd=pulumi/cicd --stack=grapl/cicd/production
git show main:pulumi/cicd/Pulumi.production.yaml
git add --verbose pulumi/cicd/Pulumi.production.yaml
pulumi config get artifacts --cwd=pulumi/cicd --stack=grapl/cicd/testing
git show main:pulumi/cicd/Pulumi.testing.yaml
git add --verbose pulumi/cicd/Pulumi.testing.yaml
git commit --message=Create new release candidate

Generated from https://buildkite.com/grapl/pipeline-infrastructure-verify/builds/2112
git --no-pager show
git push --verbose
EOF
    )

    assertEquals "${expected}" "$(recorded_commands)"
}

test_create_rc_ui_no_override() {
    (
        BUILDKITE_SOURCE=ui
        create_rc "{}" cicd/production cicd/testing
    )
    assertNotContains "Shouldn't push to git if this is a UI job" \
        "$(recorded_commands)" \
        "git push --verbose"
}

test_create_rc_ui_with_override() {
    (
        BUILDKITE_SOURCE=ui
        BREAK_GLASS_IN_CASE_OF_EMERGENCY=1
        create_rc "{}" cicd/production cicd/testing
    )
    assertContains "Should push to git even though this is a UI job" \
        "$(recorded_commands)" \
        "git push --verbose"
}
