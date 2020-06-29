#!/usr/bin/env bash

npm run build &&
cdk destroy -f --require-approval=never "*"

date
