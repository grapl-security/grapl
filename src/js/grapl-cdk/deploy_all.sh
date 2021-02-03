#!/usr/bin/env bash

set -e  # Quit upon any failure

#####
# Set `AWS_PROFILE=` for multi-profile support
if [ -z ${AWS_PROFILE} ]; then
    PROFILE_FLAG=""
else 
    PROFILE_FLAG="--profile=$AWS_PROFILE"
fi

##########
# A bunch of overhead to just get the directories right
# from https://stackoverflow.com/a/246128
THIS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
EDGE_UX_ARTIFACT_DIR="${THIS_DIR}/edge_ux_post_replace"
cd "$THIS_DIR"

mkdir -p "${EDGE_UX_ARTIFACT_DIR}"
npm run build
cdk deploy \
    --require-approval=never \
    $PROFILE_FLAG \
    --outputs-file=./cdk-output.json \
    Grapl
rm -rf "${EDGE_UX_ARTIFACT_DIR}"

mkdir -p "${EDGE_UX_ARTIFACT_DIR}"

npm run create_edge_ux_package
cdk deploy \
    --require-approval=never \
    $PROFILE_FLAG \
    EngagementUX

date
