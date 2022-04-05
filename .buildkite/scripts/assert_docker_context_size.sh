#!/usr/bin/env bash
set -euo pipefail

################################################################################
# Ensure the Docker context remains a manageable, reasonable size.
# (This helps catch missed renames that should be caught in the .dockerignore.)
################################################################################
# This can change in the future, but this seems like a reasonable starter limit
readonly ARBITRARY_SIZE_LIMIT_MB=$((300))

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
        echo "Dumped ${context_dir} context to ${temp_dir}" >&2
        cd "${temp_dir}"
        du --max-depth=0 | awk '{print $1;}'
    )
}

# anything that'll be declared as a context in docker-bake.hcl
all_docker_contexts() {
    docker buildx bake all --print |
        jq --raw-output '.target[].context' |
        sort --unique
}

mapfile -t DIRS_TO_CHECK < <(all_docker_contexts)
readonly DIRS_TO_CHECK

failed_dirs=()

for dir in "${DIRS_TO_CHECK[@]}"; do
    echo "--- Checking Docker context size of ${dir}"
    size_kb="$(get_size_of_context_kb "${dir}")"
    echo "Docker context size of ${dir} is ${size_kb}KB"
    if [[ "${size_kb}" -gt "$((ARBITRARY_SIZE_LIMIT_MB * 1024))" ]]; then
        failed_dirs+=("${dir}")
    fi
done

# ${#} means check length
if [ ${#failed_dirs[@]} -ne 0 ]; then
    echo "--- Contexts too big!"
    echo "${failed_dirs[@]}"
    echo "Exceeded size limit of ${ARBITRARY_SIZE_LIMIT_MB}MB"
    echo "Maybe you need to modify the .dockerignore?"
    exit 42
fi
