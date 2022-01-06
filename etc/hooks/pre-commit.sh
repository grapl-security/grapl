#!/usr/bin/env bash

# The sed here is to deal with any files with spaces in the names
FILES=$(git diff --cached --name-only --diff-filter=ACMR | sed 's| |\\ |g')
[ -z "$FILES" ] && exit 0

# Prettify all selected files
# (--owners-not-found-behavior=ignore will skip changed files that are not Pants-managed)
echo "$FILES" | xargs ./pants --owners-not-found-behavior=ignore fmt

# Add back the modified files to staging
echo "$FILES" | xargs git add

exit 0

# This runs pants formatting and adds it to the commit
#./pants --changed-since=HEAD fmt
