#!/bin/bash

set -euo pipefail

# Run rustfmt on our code. Suitable for use in editor integrations as well as in
# CI.
#

# By setting this environment variable (as opposed to using the `+toolchain`
# syntax in the `cargo` command), we can automatically install the given
# toolchain if it doesn't already exist on the system. It'll take a while to
# download the first time, but thereafter, it will be quick!
#
# Note that you will need your Rustup profile set to something higher
# than "minimal" (e.g., "default"), since `rustup` isn't included in
# "minimal".
export RUSTUP_TOOLCHAIN="nightly-2021-01-20"

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
