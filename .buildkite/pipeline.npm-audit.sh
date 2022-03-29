#!/usr/bin/env bash

# Dynamically generate a pipeline for running npm audit. It's only
# dynamic because we need a way to control the `soft_fail` behavior.
#
# If we're running in the context of the `grapl/npm-audit` pipeline
# (on a weekly schedule) we want to fail, because then we can get
# notified in Slack, etc.
#
# If we're running elsewhere, such as `grapl/verify` or `grapl/merge`,
# then we want to be able to fail this job without failing the whole
# pipeline; in that scenario, it's purely an advisory job.
#
# Absent a dynamically-generated pipeline like this, there isn't
# really a good way to achieve this without a lot of duplication.

set -euo pipefail

if [ "${BUILDKITE_PIPELINE_NAME}" == "grapl/npm-audit" ]; then
    soft_fail="false"
else
    soft_fail="true"
fi

cat << EOF
---
steps:
  - group: ":lock_with_ink_pen: Dependency Audits"
    steps:
      - label: ":nodejs: npm audit"
        command:
          - cd src/js/graphql_endpoint
          - npm audit
        plugins:
          - docker#v3.8.0:
              image: "node:17-alpine"
        soft_fail: ${soft_fail}
        agents:
          queue: beefy
EOF
