# Git Hooks

This directory houses optional but recommended git hooks that are specific to
the grapl repo. In particular, these should be local only, not global.

## Enabling git hooks

This should be automatically enabled as part of the
`etc/chromeos/setup_chromeos.sh` script. To manually enable a specific git hook
run

```
GIT_ROOT=$(git rev-parse --show-toplevel)
ln --symbolic --relative "$GIT_ROOT/etc/hooks/pre-commit.sh" "$GIT_ROOT/.git/hooks/pre-commit"
```

## Creating new git hooks

When creating a new git hook:

1. Make the file executable (and committed to git as executable)
2. Update the chromeos installs script to add the new git hook inside the install_git_hooks function
