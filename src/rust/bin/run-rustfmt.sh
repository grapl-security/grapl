#!/bin/bash

set -euo pipefail

# Run rustfmt on our code. Suitable for use in editor integrations as well as in
# CI.
#

if [ ! -t 0 ]; then
    # If standard input is open, we assume we're running in the
    # context of an editor integration that is passing the contents of
    # an individual file on standard input and writing formatted code
    # back to standard output.
    rustfmt < /dev/stdin
else
    # If there is no standard input, then assume we're running in CI
    # or in the terminal. In that case, just check to see if
    # everything is formatted properly without actually changing
    # anything.
    cargo fmt --all -- --check
fi
