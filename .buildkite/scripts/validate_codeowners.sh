#!/usr/bin/env bash

# Performs some validations of our CODEOWNERS file.
#
# In particular, we check to see that the syntax is correct, and there
# are no invalid rules (e.g., rules that don't match any files).
#
# We also check to see which files are "unloved" (i.e., unclaimed and
# owned by nobody.)

set -euo pipefail

echo -e "--- :npm: Installing 'github-codeowners' package"
npm install --global github-codeowners

echo -e "--- :octocat::sleuth_or_spy: Validating CODEOWNERS file"
# Unfortunately, validation failures don't change the exit code, and
# are output to stderr :/
violations=$(github-codeowners validate 2>&1)
if [ -n "${violations}" ]; then
    echo "${violations}"
    exit 1
fi

echo -e "--- :octocat::broken_heart: Finding 'unloved' files"
# Again, violations don't change the exit code
unloved=$(github-codeowners audit --only-git --unloved)
if [ -n "${unloved}" ]; then
    echo "+++ :hurtrealbad: Found unowned files; please update .github/CODEOWNERS!"
    echo "${unloved}"
    exit 2
fi
