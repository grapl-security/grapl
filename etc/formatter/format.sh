#!/bin/bash
set -u
shopt -s globstar # ** now actually works

mode=""

while (("$#")); do
    case "$1" in
        -c | --check | --ci)
            mode="check"
            shift
            ;;
        -u | --update)
            mode="update"
            shift
            ;;
        -h | --help)
            mode="help"
            shift
            ;;
    esac
done

printHelp() {
    cat >&2 << EOF

    Usage: $0 <OPTIONS>

    Options:

    -c|--check|--ci: Check the formatting of all js/ts/md code. Use
    this in CI jobs. If no other options are given, this is the
    default behavior.

    -h|--help: Print this help message.

    -u|--update: Format all js/ts/md code. Use this after updating
    the nightly version of js/ts used for formatting, updating
    configuration options, or any other time you just want to make
    sure all the code is up to date.
EOF
    exit 1
}

prettier_arg=""
if [ "${mode}" == "check" ]; then
    prettier_arg="--check"
elif [ "${mode}" == "update" ]; then
    prettier_arg="--write"
elif [ "${mode}" == "help" ]; then
    printHelp
else
    printHelp
fi

# As specified in `docker-compose.formatter.yml`
readonly repo_root="/mnt/grapl_repo_rw"

echo "--- Prettier Typescript"
prettier \
    --config prettierrc.toml \
    ${prettier_arg} \
    ${repo_root}/src/js/graphql_endpoint/**/*.ts \
    ${repo_root}/src/js/engagement_view/src/**/*.ts \
    ${repo_root}/src/js/engagement_view/src/**/*.tsx

# Slightly different config for yaml
echo "--- Prettier YAML"
prettier \
    --config prettierrc-yaml.toml \
    ${prettier_arg} \
    ${repo_root}/**/*.yml \
    ${repo_root}/**/*.yaml \
    ${repo_root}/.buildkite/**/*.yml \
    ${repo_root}/.github/**/*.yml

# No config for markdown
echo "--- Prettier Markdown"
prettier \
    ${prettier_arg} \
    --prose-wrap always \
    --print-width 80 \
    ${repo_root}"/{,!(**/(target|*venv)/**)}**/*.md"

# No config for markdown
echo "--- Prettier TOML"
prettier \
    ${prettier_arg} \
    --prose-wrap always \
    --print-width 80 \
    ${repo_root}"/{,!(**/(target|*venv)/**)}**/*.toml"
