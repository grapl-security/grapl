#!/usr/bin/env bash

set -e  # Quit upon any failure

if [ -z ${PROFILE} ]; then
    PROFILE_FLAG=""
else 
    PROFILE_FLAG="--profile=$PROFILE"
fi

##########
# A bunch of overhead to just get the directories right
# from https://stackoverflow.com/a/246128
THIS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd $THIS_DIR

npm run build
cdk deploy \
    --require-approval=never \
    $PROFILE_FLAG \
    --outputs-file=./cdk-output.json \
    Grapl
rm -rf ./edge_ux_package
npm run create_edge_ux_package
cdk synth \
    $PROFILE_FLAG
cdk deploy \
    --require-approval=never \
    $PROFILE_FLAG \
    EngagementUX

date
