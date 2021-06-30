#!/usr/bin/env bash

# Generate a version string consisting of the UTC commit timestamp and
# short git SHA of the current git commit.
timestamp_and_sha_version() {
    # Need to set TZ along with the `format-local` date formatting
    # flag to transform the date to a UTC time
    #
    # %ad is the author date
    # %h is the short git SHA
    TZ=UTC git show \
        --no-patch \
        --pretty="format:%ad-%h" \
        --date="format-local:%Y%m%d%H%M%S"
}
