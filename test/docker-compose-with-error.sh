#!/bin/bash

set -e

usage() { 
    echo 'Usage: $0 [SERVICES]' 1>&2
    echo
    echo 'This script calls into `docker-compose up` and checks the exit code' 1>&2
    echo 'of each container upon exit. If any container exit code is non-zero,' 1>&2
    echo 'this script will exit non-zero.' 1>&2
    echo
    echo 'Use Compose environment variables, such as COMPOSE_FILE and COMPOSE_PROJECT_NAME,' 1>&2
    echo 'for directing docker-compose.' 1>&2
    exit 1
}

# We'll pass SERVICES to the end of docker-compose up to allow users to specify
# specific services to run, instead of all of them, which is the default behavior.
SERVICES="$@"

# Execute the 'up'
docker-compose up --force-recreate ${SERVICES}

# check for container exit codes other than 0
EXIT_CODE=0
ALL_TESTS=$(docker-compose ps --quiet ${SERVICES})
for test in $ALL_TESTS; do
    docker inspect -f "{{ .State.ExitCode }}" $test | grep -q ^0;
    if [ $? -ne 0 ]; then 
        EXIT_CODE=$?;
        break
    fi
done

exit $EXIT_CODE