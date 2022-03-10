#!/usr/bin/env bash

set -euo pipefail

# Automatically configure AWS SSO for all the accounts you have
# access to.
#
# Note that `AWS_DEFAULT_SSO_START_URL` and `AWS_DEFAULT_SSO_REGION`
# appear to be specific to `aws-sso-util`, not to the AWS CLI itself.
#
# These values can be set in your Gitpod Variables:
# https://gitpod.io/variables

# The region that we'll be logging into for each account.
readonly region="us-east-1"

aws-sso-util configure populate \
    --sso-start-url="${AWS_DEFAULT_SSO_START_URL}" \
    --sso-region="${AWS_DEFAULT_SSO_REGION}" \
    --components=account_name \
    --region="${region}" \
    --components=account_name \
    --trim-account-name="'" \
    --trim-account-name="_"
