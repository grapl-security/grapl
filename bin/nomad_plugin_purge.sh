#!/usr/bin/env bash

# This script is meant to be used to clear dead copies of the grapl-plugin job in Nomad.

# Usage:
# Make sure there is an open tunnel to Nomad (typically via ./bin/aws/ssm_nomad_server.sh in a separate tab)
# Then run this script ./bin/nomad_plugin_purge.sh

plugin_namespaces=$(nomad job stop -namespace=* -purge grapl-plugin 2>&1 | tail -n +3 | awk '{ if ($5 == "dead") { print $2 } }')

for namespace in $plugin_namespaces; do
    echo "Purging grapl-plugin in namespace $namespace"
    nomad job stop -namespace="$namespace" -purge grapl-plugin
done

# Ensure that nomad cleans up after itself
nomad system gc
