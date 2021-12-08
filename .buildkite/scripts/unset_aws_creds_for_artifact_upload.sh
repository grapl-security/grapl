# We have to unset the AWS credentials injected by the
# assume-role plugin if we're going to subsequently upload the
# file to our bucket :(

# More info at https://github.com/grapl-security/grapl/pull/1277

unset AWS_ACCESS_KEY_ID
unset AWS_SECRET_ACCESS_KEY
unset AWS_SESSION_TOKEN
