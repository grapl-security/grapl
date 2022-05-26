#!/usr/bin/env bash
# This script is used when a pulumi state has pending operations because pulumi was interrupted mid-run
# It removes all pending operations and refreshes the stack so you have a clean and updated state.

# Reasons you might end up with pending operations in your pulumi state:
# * aws token expired mid-run
# * lost internet mid-run
# * linux went into standby mode when you grabbed coffee/food/etc
# * control-C'd pulumi mid-run

# Usage:
# ./pulumi/bin/pulumi_fix_stack.sh pulumi/grapl

set -euo pipefail

readonly PULUMI_PROJECT_PATH="$1"

pulumi stack export --cwd "$PULUMI_PROJECT_PATH" > "$PULUMI_PROJECT_PATH/stack.json"
jq 'del(.deployment.pending_operations)' "$PULUMI_PROJECT_PATH/stack.json" > "$PULUMI_PROJECT_PATH/stack_fixed.json"
pulumi stack import --cwd "$PULUMI_PROJECT_PATH" --file "stack_fixed.json"
# clean up
rm "$PULUMI_PROJECT_PATH/stack.json"
rm "$PULUMI_PROJECT_PATH/stack_fixed.json"
# refresh
pulumi refresh --cwd "$PULUMI_PROJECT_PATH" --yes
