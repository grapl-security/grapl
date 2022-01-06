#!/usr/bin/env bash

# The sed here is to deal with any files with spaces in the names
FILES=$(./pants list --changed-since=HEAD)
[ -z "$FILES" ] && exit 0

# Prettify all selected files
echo "$FILES" | xargs ./pants fmt

# Add back the modified files to staging
echo "$FILES" | xargs git add

exit 0

# This runs pants formatting and adds it to the commit
#./pants --changed-since=HEAD fmt
