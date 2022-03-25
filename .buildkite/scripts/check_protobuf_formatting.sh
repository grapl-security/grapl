#!/usr/bin/env sh

# NOTE: This runs in the bufbuild/buf container, which (as of version
# 1.2.1, at least) does *not* contain `bash`, so we *must* use `sh`
# instead.

# If `buf` would exit with a non-zero code when a diff is found, we
# wouldn't need this script at all, and could just invoke `buf format
# --diff` in the container directly. Alas, that is not the case, so we
# get to do a bit of shell indirection.

output="$(buf format --diff)"
if [ -n "${output}" ]; then
    echo "${output}"
    echo
    echo "Protobuf files are not properly formatted!"
    exit 1
fi
