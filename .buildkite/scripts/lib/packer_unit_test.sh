#!/usr/bin/env bash
# This test runs with `make test-unit-python` (poorly named).

set -u
# Probably don't want -e so you can check return codes

oneTimeSetUp() {
    source .buildkite/scripts/lib/packer.sh
}

test_build_fake_image_name() {
    stdout=$(build_ami "fakeimage")
    exitcode=$?
    assertContains "That is not one of the 2 accepted image names" "${stdout}"
    assertNotEquals 0 $exitcode
}

# Mock out buildkite agent. Gross
buildkite-agent() {
    # Expected to be called like:
    # buildkite-agent artifact upload <file>
    #                    $1       $2    $3
    local -r filename=$3
    if [ -f "$filename" ]; then
        echo "$filename exists."
    else
        echo "$filename does not exist."
        exit 42
    fi

}

test_upload_manifest() {
    # ultimately, i just wanna test that we send the right filename to buildkite upload
    local -r image_name="fakeimage"
    echo "im-a-manifest" > "${image_name}.packer-manifest.json"

    stdout=$(upload_manifest ${image_name})
    exitcode=$?

    assertEquals 0 $exitcode
    # `upload_manifest` conveniently `cat`s our file
    assertContains "im-a-manifest" "${stdout}"
}
