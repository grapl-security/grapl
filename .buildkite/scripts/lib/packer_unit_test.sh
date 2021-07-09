#!/usr/bin/env bash
# This test runs with `make test-unit-python` (poorly named).

set -u
# Probably don't want -e so you can check return codes

oneTimeSetUp() {
    source .buildkite/scripts/lib/packer.sh
}

test_build_fake_image_name() {
    local -r stdout=$(build_ami "fakeimage")
    assertEquals "Unknown PACKER_IMAGE_NAME fakeimage" "${stdout}"
}

# Mock out buildkite agent. Gross
buildkite-agent() {
    # Expected to be called like:
    # buildkite-agent artifact upload <file>
    #                    $1       $2    $3
    local -r filename=$3
    echo "Filename is ${filename}"

    if [ -f "$filename" ]; then
        echo "$filename exists."
    else
        echo "$filename does not exist."
        exit 43
    fi

}
alias buildkite-agent=mock_buildkite_agent

test_upload_manifest() {
    local -r image_name="fakeimage"
    echo "im-a-manifest" > "${image_name}.packer-manifest.json"

    stdout=$(upload_manifest ${image_name})
    exitcode=$?

    assertEquals 0 $exitcode
    # `upload_manifest` conveniently `cat`s our file
    assertContains "im-a-manifest" "${stdout}"
}

test_upload_manifest_file_doesnt_exist() {
    local -r image_name="fakeimage2"
    # no touch

    stdout=$(upload_manifest ${image_name})
    exitcode=$?

    # A failed buildkite upload results in 43
    assertEquals 43 ${exitcode}
    assertContains "fakeimage2.packer-manifest.json does not exist" "${stdout}"
}
