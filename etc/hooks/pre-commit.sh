#!/usr/bin/env bash

# The sed here is to deal with any files with spaces in the names
FILES=$(git diff --cached --name-only --diff-filter=ACMR | sed 's| |\\ |g')
# fast exit if no files were updated
[ -z "$FILES" ] && exit 0

# Prettify all changed files managed by Pants
./pants fmt --changed-since=HEAD

# Add back the modified files to staging
echo "$FILES" | xargs git add --update

exit 0
