#!/bin/bash

set -e
trap compose_stop EXIT

compose_stop() {
    docker-compose ${FILE_ARGS} --project-name "$p" stop
}

usage() { 
    echo 'Usage: $0 -p <project-name> -f docker-compose1.yml [SERVICES]' 1>&2
    echo
    echo 'This script calls into `docker-compose up` with the supplied arguments' 1>&2
    echo 'and checks the exit code of each container upon exit. If any container' 1>&2
    echo 'exit code is non-zero, this script will exit non-zero.' 1>&2
    echo
    echo 'Options:' 1>&2
    echo '    -f	Path to compose file. Can be passed multiple times.' 1>&2
    echo '    -p	Project name.' 1>&2
    echo '    -t	Target (if you only want to execute one thing).' 1>&2
    exit 1
}

while getopts "hf:p:t:" arg; do
    case $arg in
        f)
            FILE_ARGS+="-f ${OPTARG} "
            ;;
        t)
            TARGET_ARGS="${OPTARG} "
            ;;
        p)
            p=${OPTARG}
            ;;
        h) # Show help
            usage
            ;;
    esac
done

if [ -z "${FILE_ARGS}" ] || [ -z "${p}" ]; then
    usage
fi

if [ -z "${TARGET_ARGS}" ]; then
    TARGET_ARGS=""
fi

shift $(($OPTIND - 1))
SERVICES="$@"

# Execute the 'up'
docker-compose ${FILE_ARGS} --project-name "$p" up --force-recreate ${SERVICES} ${TARGET_ARGS}

# check for container exit codes other than 0
EXIT_CODE=0
ALL_TESTS=$(docker-compose ${FILE_ARGS} --project-name "$p" ps -q ${SERVICES})
for test in $ALL_TESTS; do
    docker inspect -f "{{ .State.ExitCode }}" $test | grep -q ^0;
    if [ $? -ne 0 ]; then 
        EXIT_CODE=$?;
        break
    fi
done

compose_stop

exit $EXIT_CODE