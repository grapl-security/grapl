#!/usr/bin/env bash

# Dynamically generate a pipeline for running cargo udeps. It's only
# dynamic because we need a way to control the `soft_fail` behavior.
#
# If we're running in the context of the `grapl/cargo-udeps` pipeline
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

# Hacky way to extract a value from a TOML file T_T
#
# This at least automatically keeps things in sync with with our Rust
# toolchain.
rust_version="$(grep channel src/rust/rust-toolchain.toml | sed -E 's/channel = "(.*)"/\1/g')"
readonly rust_version

if [ "${BUILDKITE_PIPELINE_NAME}" == "grapl/cargo-udeps" ]; then
    soft_fail="false"
else
    soft_fail="true"
fi

# `cargo-udeps` must run under the nightly toolchain. We must take
# care to install the nightly toolchain with the default profile to
# ensure that we also download the `rustfmt` component for nightly as
# well.
#
# We do this because `tonic_build` is currently compiled using the
# `rustfmt` feature (enabled by default), which formats the generated
# code and makes for better error messages.
#
# See
# https://docs.rs/tonic-build/latest/tonic_build/index.html#features
# for details.
cat << EOF
---
steps:
  - label: ":rust: cargo udeps"
    command:
      - cd src/rust
      - cargo install cargo-udeps --locked
      - rustup toolchain install nightly --profile=default
      - cargo +nightly udeps --all-features --all-targets
    plugins:
      - docker#v3.8.0:
          image: "rust:${rust_version}"
    soft_fail: ${soft_fail}
    agents:
      queue: beefy
EOF
