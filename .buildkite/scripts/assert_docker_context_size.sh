#!/usr/bin/env bash
set -euo pipefail

################################################################################
# Ensure the Docker context remains a manageable, reasonable size.
# (This helps catch missed renames that should be caught in the .dockerignore.)
################################################################################
readonly GRAPL_ROOT="${PWD}"

get_size_of_context_kb() {
    # Based on https://stackoverflow.com/a/61821807
    local -r context_dir="${1}"
    temp_dir="$(mktemp --directory)"
    (
        cd "${context_dir}"
        printf 'FROM scratch\nCOPY . /' | DOCKER_BUILDKIT=1 docker build \
            --file - \
            --output "${temp_dir}" \
            .
        echo >&2 "Dumped context to ${temp_dir}"
        cd "${temp_dir}"
        du --max-depth=0 | awk '{print $1;}'
    )
}

# This can change in the future, but this seems like a reasonable starter limit
readonly ARBITRARY_SIZE_LIMIT_KB=$((300 * 1024))

# anything that'll be declared as a context in docker-bake.hcl
readonly DIRS_TO_CHECK=(
    "${GRAPL_ROOT}"
    "${GRAPL_ROOT}/src"
    "${GRAPL_ROOT}/src/js/graphql_endpoint"
    "${GRAPL_ROOT}/localstack"
    "${GRAPL_ROOT}/postgres"
)

for dir in "${DIRS_TO_CHECK[@]}"; do
    size="$(get_size_of_context_kb "${dir}")"
    echo "--- Docker context size of ${dir} is ${size}kb"
    if [[ "${size}" -gt "${ARBITRARY_SIZE_LIMIT_KB}" ]]; then
        echo "That's too big! Maybe you need to modify the Dockerignore?"
        exit 42
    fi
done
