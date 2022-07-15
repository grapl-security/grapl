#!/usr/bin/env bash

# Dynamically generate a pipeline for running cargo audit. It's only
# dynamic because we need a way to control the `soft_fail` behavior.
#
# If we're running in the context of the `grapl/cargo-audit` pipeline
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

rust_version="$(./build-support/rust_version.sh)"
readonly rust_version

if [ "${BUILDKITE_PIPELINE_NAME}" == "grapl/cargo-audit" ]; then
    soft_fail="false"
else
    soft_fail="true"
fi

cat << EOF
---
steps:
  - group: ":lock_with_ink_pen: Dependency Audits"
    steps:
      - label: ":rust: cargo audit"
        command:
          - cd src/rust
          - cargo install cargo-audit
          - cargo audit
        plugins:
          - docker#v3.8.0:
              image: "rust:${rust_version}"
        soft_fail: ${soft_fail}
        agents:
          queue: beefy
EOF
