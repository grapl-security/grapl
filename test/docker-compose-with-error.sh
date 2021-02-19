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
    exit 1
}

while getopts "hf:p:" arg; do
    case $arg in
        f)
            FILE_ARGS+="-f ${OPTARG} "
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

shift $(($OPTIND - 1))
SERVICES="$@"

docker-compose ${FILE_ARGS} --project-name "$p" up --force-recreate ${SERVICES}

# check for container exit codes other than 0
EXIT_CODE=0
for test in $(docker-compose ${FILE_ARGS} --project-name "$p" ps -q ${SERVICES}); do
    docker inspect -f "{{ .State.ExitCode }}" $test | grep -q ^0;
    if [ $? -ne 0 ]; then 
        EXIT_CODE=$?;
        break
    fi
done

compose_stop

exit $EXIT_CODE