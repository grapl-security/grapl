#!/bin/bash

set -e

usage() { echo "Usage: $0 -f docker-compose.yml" 1>&2; exit 1; }

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

if [ -z "${FILE_ARGS}" ] ; then
    usage
fi

shift $(($OPTIND - 1))
SERVICES="$@"

docker-compose ${FILE_ARGS} --project-name "$p" up --force-recreate ${SERVICES}

# check for container exit codes other than 0
for test in $(docker-compose ${FILE_ARGS} --project-name "$p" ps -q ${SERVICES}); do
    docker inspect -f "{{ .State.ExitCode }}" $test | grep -q ^0;
    if [ $? -ne 0 ]; then 
        exit 1; 
    fi
done