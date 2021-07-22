# requirements.txt

Declare all your dependencies. Things should probably only be pinned in here if
they need to be for some reason (and that reason should be documented in a
comment).

# constraints.txt

Generated from `requirements.txt` by using
`./build-support/manage_virtualenv.sh regenerate-constraints` Automatically pins
versions. These are the libs and versions actually used in pants builds.
