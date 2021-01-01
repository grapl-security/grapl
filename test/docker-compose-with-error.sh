#!/bin/bash

usage() { echo "Usage: $0 -f docker-compose.yml" 1>&2; exit 1; }

while getopts "hf:" arg; do
    case $arg in
        f)
            f=${OPTARG}
            ;;
        h | *) # Show help
            usage
            ;;
    esac
done

if [ -z $f ] ; then
    usage
fi

docker-compose -f "$f" up

# check for container exit codes other than 0
for test in $(docker-compose -f "$f" ps -q); do
    docker inspect -f "{{ .State.ExitCode }}" $test | grep -q ^0;
    if [ $? -ne 0 ]; then 
        exit 1; 
    fi
done