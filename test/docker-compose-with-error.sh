#!/bin/bash

# You can pass TARGETS to this script to specify
# which services to run, instead of all of them, which is the default behavior.
TARGETS="${TARGETS}"

set -e

usage() { 
    echo 'Usage: [TARGETS="service1 service2"] $0' 1>&2
    echo
    echo 'This script calls into `docker-compose up` and checks the exit code' 1>&2
    echo 'of each container upon exit. If any container exit code is non-zero,' 1>&2
    echo 'this script will exit non-zero.' 1>&2
    echo
    echo 'Use Compose environment variables, such as COMPOSE_FILE and COMPOSE_PROJECT_NAME,' 1>&2
    echo 'for directing docker-compose.' 1>&2
    exit 1
}

# Execute the 'up'
docker-compose up --force-recreate ${TARGETS}

# check for container exit codes other than 0
EXIT_CODE=0
ALL_TESTS=$(docker-compose ps --quiet ${TARGETS})
for test in $ALL_TESTS; do
    docker inspect -f "{{ .State.ExitCode }}" $test | grep -q ^0;
    if [ $? -ne 0 ]; then 
        EXIT_CODE=$?;
        break
    fi
done

exit $EXIT_CODE