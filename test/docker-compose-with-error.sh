#!/bin/bash

set -e

usage() { echo "Usage: $0 -f docker-compose.yml" 1>&2; exit 1; }

while getopts "hf:p:" arg; do
    case $arg in
        f)
            f=${OPTARG}
            ;;
        p)
            p=${OPTARG}
            ;;
        h) # Show help
            usage
            ;;
    esac
done

if [ -z $f ] ; then
    usage
fi

shift $(($OPTIND - 1))
SERVICES="$@"

docker-compose -f "$f" -p "$p" up --force-recreate ${SERVICES}

# check for container exit codes other than 0
for test in $(docker-compose -f "$f" -p "$p" ps -q ${SERVICES}); do
    docker inspect -f "{{ .State.ExitCode }}" $test | grep -q ^0;
    if [ $? -ne 0 ]; then 
        exit 1; 
    fi
done