#!/usr/bin/env bash

oneTimeSetUp() {
    # shellcheck source-path=SCRIPTDIR
    source "$(dirname "${BASH_SOURCE[0]}")/pulumi.sh"
}

test_split_project() {
    assertEquals "project" "$(split_project "project/stack")"
}

test_split_stack() {
    assertEquals "stack" "$(split_stack "project/stack")"
}

test_fully_qualified_stack_name() {
    assertEquals "grapl/cicd/production" "$(fully_qualified_stack_name cicd/production)"
}

test_project_directory() {
    assertEquals "pulumi/project" "$(project_directory "project/stack")"
    assertEquals "pulumi/foo_bar" "$(project_directory "foo-bar/stack")"
    assertEquals "pulumi/foo_bar_baz_quux" "$(project_directory "foo-bar-baz-quux/stack")"
    assertEquals "pulumi/boo_baz" "$(project_directory "boo_baz/stack")"
}

test_stack_file_path() {
    assertEquals "pulumi/foo_bar/Pulumi.testing.yaml" "$(stack_file_path "foo-bar/testing")"
}
