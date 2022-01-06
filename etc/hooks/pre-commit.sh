#!/usr/bin/env bash

# Prettify all changed files managed by Pants
./pants fmt --changed-since=HEAD

# Add back the modified files to staging
git add --update

exit 0
