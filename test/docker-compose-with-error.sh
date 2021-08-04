#!/bin/bash
# shellcheck disable=SC2086

# This script will honor the TARGETS environment variable to specify
# which services to run, instead of all of them, which is the default
# behavior.
TARGETS="${TARGETS:-}"

set -eu

usage() {
    echo "Usage: [TARGETS=\"service1 service2\"] $0" 1>&2
    echo
    echo "This script calls into $(docker-compose up) and checks the exit code" 1>&2
    echo "of each container upon exit. If any container exit code is non-zero," 1>&2
    echo "this script will exit non-zero." 1>&2
    echo
    echo "Use Compose environment variables, such as COMPOSE_FILE and COMPOSE_PROJECT_NAME," 1>&2
    echo "for directing docker-compose." 1>&2
    exit 1
}

# Execute the 'up'
docker-compose up \
    --force-recreate \
    --always-recreate-deps \
    --renew-anon-volumes \
    ${TARGETS}

# check for container exit codes other than 0
EXIT_CODE=0
ALL_TESTS=$(docker-compose ps --quiet ${TARGETS})
for test in $ALL_TESTS; do
    test_exit_code=$(docker inspect -f "{{ .State.ExitCode }}" "${test}")
    if [[ ${test_exit_code} -ne 0 ]]; then
        EXIT_CODE=$test_exit_code
        break
    fi
done

echo "docker-compose-with-error: exit code $EXIT_CODE"
exit $EXIT_CODE
