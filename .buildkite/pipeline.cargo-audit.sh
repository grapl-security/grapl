#!/usr/bin/env bash

# Dynamically generate a pipeline for running cargo audit. It's only
# dynamic because we need a way to control the `soft_fail` behavior.
#
# If we're running in the context of the `grapl/verify` pipeline, we
# want to be able to fail this job without failing the whole pipeline;
# in that scenario, it's purely an advisory job.
#
# If we're running elsewhere, though, such as in our own dedicated
# pipeline on a scheduled basis, we *do* want to fail, because then we
# can get notified in Slack, etc.
#
# Absent a dynamically-generated pipeline like this, there isn't
# really a good way to achieve this without a lot of duplication.

set -euo pipefail

if [ "${BUILDKITE_PIPELINE_NAME}" == "grapl/verify" ]; then
    soft_fail="true"
else
    soft_fail="false"
fi

cat <<EOF
---
steps:
  - label: ":rust: cargo audit"
    command:
      - cd src/rust
      - cargo install cargo-audit
      - cargo audit
    plugins:
      - docker#v3.8.0:
          image: "rust:1.51.0"
    soft_fail: ${soft_fail}
    agents:
      queue: beefy
EOF
