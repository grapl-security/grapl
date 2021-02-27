#!/usr/bin/env bash

npm run build &&
cdk destroy -f --require-approval=never "*"

# Clear out the cdk-out assets, which are never cleaned by themselves
# https://github.com/aws/aws-cdk/issues/2869
cd cdk-out
find . -name 'asset.*' -print0 | xargs -0 rm -rf

date
