shell_library()

# Make the `.jq` filters available at test time
files(
    name="test_fixtures",
    sources=[
        "*.jq",
        "example_packer_artifact.json",
    ]
)

shunit2_tests(
    name="tests",
    dependencies=[
        ":test_fixtures",
    ]
)
