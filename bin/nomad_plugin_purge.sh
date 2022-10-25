#!/usr/bin/env bash

# This script is meant to be used to clear dead copies of the grapl-plugin job in Nomad.

# Usage:
# Make sure there is an open tunnel to Nomad (typically via ./bin/aws/ssm_nomad_server.sh in a separate tab)
# Then run this script ./bin/nomad_plugin_purge.sh

# `nomad job stop -namespace='*' -purge grapl-plugin` returns the following to stderr when there are multiple plugins
#Prefix matched multiple jobs
#
#ID            Namespace                                    Type     Priority  Status   Submit Date
#grapl-plugin  plugin-3ee133b7-8147-41bb-8908-283e0d946197  service  50        running  2022-10-17T11:56:31-04:00
#grapl-plugin  plugin-4234e3b1-7dc0-4511-b424-186342beee5d  service  50        running  2022-10-17T11:56:30-04:00
#grapl-plugin  plugin-58213bfc-2ebb-42d5-ac99-b06b9f03187e  service  50        pending  2022-08-11T15:36:42-04:00
#grapl-plugin  plugin-9855f150-edc9-4fdb-b271-a5098661f8ad  service  50        dead     2022-10-17T11:56:30-04:00
#grapl-plugin  plugin-b5fb0641-00ec-4510-9af5-a0e3948936b7  service  50        pending  2022-08-11T15:39:12-04:00

plugin_namespaces=$(nomad job stop -namespace='*' -purge grapl-plugin 2>&1 | tail -n +3 | awk '{ if ($5 == "dead") { print $2 } }')

for namespace in $plugin_namespaces; do
    echo "Purging grapl-plugin in namespace $namespace"
    nomad job stop -namespace="$namespace" -purge grapl-plugin
done

# Ensure that nomad cleans up after itself
nomad system gc
