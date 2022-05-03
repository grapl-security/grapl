# syntax=docker/dockerfile:1.3-labs
# We use the above syntax for here documents:
# https://github.com/moby/buildkit/blob/master/frontend/dockerfile/docs/syntax.md#user-content-here-documents

FROM rust:1-slim-bullseye AS base

ARG RUST_BUILD=debug

SHELL ["/bin/bash", "-o", "errexit", "-o", "nounset", "-o", "pipefail", "-c"]

# curl, jq, and unzip are used by various commands in this Dockerfile.
# build-essential, cmake, libssl-dev, perl, pkg-config, and tcl are needed
# for building rust-rdkafka.
#
# Ignore this lint about deleteing the apt-get lists (we're caching!)
# hadolint ignore=DL3009,SC1089
RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rust-base-apt \
    apt-get update \
    && apt-get install --yes --no-install-recommends \
        curl=7.74.0-1.3+deb11u1 \
        jq=1.6-2.1 \
        unzip=6.0-26 \
    && apt-get install --yes --no-install-recommends \
        build-essential=12.9 \
        cmake=3.18.4-2+deb11u1 \
        libssl-dev=1.1.1n-0+deb11u1 \
        perl=5.32.1-4+deb11u2 \
        pkg-config=0.29.2-1 \
        tcl=8.6.11+1

# Grab a Nomad binary, which we use for parsing HCL2-with-variables into JSON:
# - in plugin-registry integration tests
# - in plugin-registry image
WORKDIR /nomad
RUN <<EOF
NOMAD_VERSION="1.2.4"
ZIP_NAME="nomad_${NOMAD_VERSION}_linux_amd64.zip"
curl --remote-name --proto '=https' --tlsv1.2 -sSf \
  "https://releases.hashicorp.com/nomad/${NOMAD_VERSION}/${ZIP_NAME}"
unzip "${ZIP_NAME}"
rm "${ZIP_NAME}"
EOF

# Install rust toolchain before copying sources to avoid unecessarily
# resinstalling on source file changes.
WORKDIR /grapl
COPY rust/rust-toolchain.toml rust/rust-toolchain.toml
WORKDIR /grapl/rust
# 'rustup show' will install components in the rust-toolchain.toml file
RUN rustup show

# copy sources
WORKDIR /grapl
COPY proto proto
COPY rust rust

WORKDIR /grapl/rust


ENV CARGO_TARGET_DIR="/grapl/rust/target"

# These variables are just to DRY up some repeated cache target
# locations. They are of our own creation, and do not hold any special
# meaning to `cargo`, `rustup`, or anything else.
ENV REGISTRY_CACHE_TARGET="${CARGO_HOME}/registry"
ENV RUSTUP_CACHE_TARGET="${RUSTUP_HOME}"

# build
################################################################################
FROM base AS build

# Hadolint appears to be confused about some of these mount targets
# hadolint ignore=SC1091
RUN --mount=type=cache,target="${CARGO_TARGET_DIR}",sharing=locked \
    --mount=type=cache,target="${REGISTRY_CACHE_TARGET}",sharing=locked \
    --mount=type=cache,target="${RUSTUP_CACHE_TARGET}",sharing=locked <<EOF
    case "${RUST_BUILD}" in
      debug)
        cargo build ;;
      release)
        cargo build --release ;;
      test)
        cargo test ;;
      *)
        echo "ERROR:  Unknown RUST_BUILD option: ${RUST_BUILD}";
        exit 1 ;;
    esac
EOF

# Copy the build outputs to location that's not a cache mount.
# TODO: switch to using --out-dir when stable: https://github.com/rust-lang/cargo/issues/6790
RUN --mount=type=cache,target="${CARGO_TARGET_DIR}",sharing=locked \
    mkdir -p /outputs && \
    find "${CARGO_TARGET_DIR}/${RUST_BUILD}" -maxdepth 1 -type f -executable -exec cp {} /outputs \;


# tarpaulin
# This target is not merged with the `build` target because the actions to run
# after cargo are different when building for tests and building the services,
# and we'd rather not save all of the Rust `target/` directory to Docker image
# if we don't have to.
################################################################################
FROM base AS tarpaulin

# These packages are required to compile cargo-tarpaulin itself.
RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rust-tarpaulin-apt \
    apt-get update \
    && apt-get install --yes --no-install-recommends \
        libssl-dev=1.1.1n-0+deb11u1 \
        pkg-config=0.29.2-1

# For test coverage reports
# Tarpaulin will recompile the sources from scratch and effectively taint build
# outputs, such that subsequent cargo build runs will need to start from
# scratch as well. For this reason we avoid mounting the cached target
# directory.
RUN --mount=type=cache,target="${REGISTRY_CACHE_TARGET}",sharing=locked \
    --mount=type=cache,target="${RUSTUP_CACHE_TARGET}",sharing=locked \
    cargo install cargo-tarpaulin


# build-test-integration
################################################################################
FROM base AS build-test-integration

# For running integration tests we're going to copy the test binaries to a new
# container base and run the directly, as opposed to running them via `cargo
# test`. Cargo will recompile tests if it thinks the test binaries in target/
# are out of date. Because we're using a mount cache when building the sources
# this directly won't be available in resulting container images. In the past
# we've `cp -a` the target directory to preserve it, but this can make for an
# increasingly large container image size, especially when the mount cache is
# has not been cleaned in a while. To find the test binaries paths we parse
# the manifest.json from the cargo build.
# https://github.com/rust-lang/cargo/issues/1924
# https://github.com/rust-lang/cargo/issues/3670

ENV RUST_INTEGRATION_TEST_FEATURES="node-identifier/integration,sqs-executor/integration,model-plugin-deployer/integration,plugin-work-queue/integration,organization-management/integration"
ENV TEST_DIR=/grapl/tests

RUN mkdir --parents "${TEST_DIR}"

# This will build the integration test binaries and parse the manifest to find
# their paths for copying later.
#
# Hadolint is confused again, at the time of this writing, SHELL *does*
# have -o pipefail set on line 9.
# hadolint ignore=DL4006
RUN --mount=type=cache,target="${CARGO_TARGET_DIR}",sharing=locked \
    --mount=type=cache,target="${REGISTRY_CACHE_TARGET}",sharing=locked \
    --mount=type=cache,target="${RUSTUP_CACHE_TARGET}",sharing=locked \
    cargo test \
        --features "${RUST_INTEGRATION_TEST_FEATURES}" \
        --no-run \
        --message-format=json \
        --test "*" | \
        jq -r "select(.profile.test == true) | .filenames[]" | \
        xargs \
          --replace="{}" \
          cp "{}" "${TEST_DIR}/"


# integration tests distribution
################################################################################
# We're unable to use one of the 'distroless' container images as a base here
# because our integration tests require zlib shared library, but we don't have
# a way of including that in the base image. With a debian image we can apt
# install as needed, but the debian image we're using has zlib already.
FROM debian:bullseye-slim AS integration-tests

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rust-tests-apt \
    apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates=20210119

# Put the Nomad binary on PATH
# so that it's available to integration test consumers of NomadCli
# hadolint ignore=DL3022
COPY --from=build /nomad/nomad /bin

COPY --from=build-test-integration /grapl/tests /tests

COPY --chmod=777 <<-"EOF" /run-tests.sh
#!/bin/bash

# This script is intended to be used for running integration tests. The exit code of this script will be that of
# that last test that had a non-zero exit code (test failure), otherwise it
# will be zero.

EXIT_STATUS=0
declare -a FAILING_TESTS=()

for test in $(find /tests -type f -executable -exec readlink -f {} \;)
do
    echo "--- Executing ${test}"
    # Redirect stderr so it's inline with stdout
    "${test}"
    exit_code=$?
    if [[ ${exit_code} -ne 0 ]]; then
        FAILING_TESTS+=("${test}")
        echo Test failed with exit code ${exit_code}
        EXIT_STATUS=${exit_code}
    fi
done

if [[ ${EXIT_STATUS} -ne 0 ]]; then
  echo ""
  echo ">>> FAILING TESTS: ${FAILING_TESTS[@]}"
  echo "   See .stderr.txt (or the Nomad stderr tab) to see additional failure backtraces."
fi
exit ${EXIT_STATUS}

EOF

ENTRYPOINT [ "/run-tests.sh" ]


# build-test-integration-new
################################################################################
FROM base AS build-test-integration-new

# For running integration tests we're going to copy the test binaries to a new
# container base and run the directly, as opposed to running them via `cargo
# test`. Cargo will recompile tests if it thinks the test binaries in target/
# are out of date. Because we're using a mount cache when building the sources
# this directly won't be available in resulting container images. In the past
# we've `cp -a` the target directory to preserve it, but this can make for an
# increasingly large container image size, especially when the mount cache is
# has not been cleaned in a while. To find the test binaries paths we parse
# the manifest.json from the cargo build.
# https://github.com/rust-lang/cargo/issues/1924
# https://github.com/rust-lang/cargo/issues/3670

ENV RUST_INTEGRATION_TEST_FEATURES="pipeline-ingress/new_integration_tests,plugin-registry/new_integration_tests"
ENV TEST_DIR=/grapl/tests

# This will build the integration test binaries and parse the manifest to find
# their paths for copying later.
#
# Hadolint is confused again, at the time of this writing, SHELL *does*
# have -o pipefail set on line 9.
# hadolint ignore=DL4006
RUN mkdir --parents "${TEST_DIR}"
# hadolint ignore=DL4006
RUN --mount=type=cache,target="${CARGO_TARGET_DIR}",sharing=locked \
    --mount=type=cache,target="${REGISTRY_CACHE_TARGET}",sharing=locked \
    --mount=type=cache,target="${RUSTUP_CACHE_TARGET}",sharing=locked \
    cargo test \
        --features "${RUST_INTEGRATION_TEST_FEATURES}" \
        --no-run \
        --message-format=json \
        --test "*" | \
        jq -r "select((.profile.test == true) and (.features[] | contains(\"new_integration_tests\"))) | .filenames[]" | \
        xargs \
          --replace="{}" \
          cp "{}" "${TEST_DIR}/"


# "new" integration tests distribution
################################################################################
# We're unable to use one of the 'distroless' container images as a base here
# because our integration tests require zlib shared library, but we don't have
# a way of including that in the base image. With a debian image we can apt
# install as needed, but the debian image we're using has zlib already.
FROM debian:bullseye-slim AS integration-tests-new

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rust-tests-apt \
    apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates=20210119

# Put the Nomad binary on PATH
# so that it's available to integration test consumers of NomadCli
# hadolint ignore=DL3022
COPY --from=build /nomad/nomad /bin

COPY --from=build-test-integration-new /grapl/tests /tests

COPY --chmod=777 <<-"EOF" /run-tests.sh
#!/bin/bash

# This script is intended to be used for running integration tests. The exit code of this script will be that of
# that last test that had a non-zero exit code (test failure), otherwise it
# will be zero.

EXIT_STATUS=0
declare -a FAILING_TESTS=()

for test in $(find /tests -type f -executable -exec readlink -f {} \;)
do
    echo "--- Executing ${test}"
    # Redirect stderr so it's inline with stdout
    "${test}"
    exit_code=$?
    if [[ ${exit_code} -ne 0 ]]; then
        FAILING_TESTS+=("${test}")
        echo Test failed with exit code ${exit_code}
        EXIT_STATUS=${exit_code}
    fi
done

if [[ ${EXIT_STATUS} -ne 0 ]]; then
  echo ""
  echo ">>> FAILING TESTS: ${FAILING_TESTS[@]}"
  echo "   See .stderr.txt (or the Nomad stderr tab) to see additional failure backtraces."
fi
exit ${EXIT_STATUS}

EOF

ENTRYPOINT [ "/run-tests.sh" ]


# images for running services
################################################################################
# More information about the base image used here can be found at:
# https://github.com/GoogleContainerTools/distroless/blob/main/cc/README.md.
# For debugging see: https://github.com/GoogleContainerTools/distroless#debug-images

# NOTE: we're using the debug containers at the moment so we have a
# shell; this lets us inject our Pulumi outputs in Local Grapl. If
# not for that, we could use the standard non-debug images.
FROM gcr.io/distroless/cc:debug AS rust-dist

USER nonroot

##### analyzer-dispatcher
FROM rust-dist AS analyzer-dispatcher-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/analyzer-dispatcher /
ENTRYPOINT ["/analyzer-dispatcher"]

##### generic-subgraph-generator
FROM rust-dist AS generic-subgraph-generator-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/generic-subgraph-generator /
ENTRYPOINT ["/generic-subgraph-generator"]

##### graph-merger
FROM rust-dist AS graph-merger-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/graph-merger /
ENTRYPOINT ["/graph-merger"]

##### plugin-work-queue
FROM rust-dist AS plugin-work-queue-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/plugin-work-queue /
ENTRYPOINT ["/plugin-work-queue"]

##### plugin-registry
FROM rust-dist AS plugin-registry-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/plugin-registry /
# Put the Nomad binary on PATH for NomadCli class
# hadolint ignore=DL3022
COPY --from=build /nomad/nomad /bin
ENTRYPOINT ["/plugin-registry"]

##### plugin-bootstrap
FROM rust-dist AS plugin-bootstrap-deploy

# hadolint ignore=DL3022
COPY --from=build /outputs/plugin-bootstrap /
ENTRYPOINT ["/plugin-bootstrap"]

##### plugin-bootstrap-init.
# Not "-deploy" since we copy it to dist/.
FROM scratch AS plugin-bootstrap-init-output

# hadolint ignore=DL3022
COPY --from=build /outputs/plugin-bootstrap-init /
# Just to clarify: we're copying these .service files from the repository,
# through Docker, and then back out to the dist directory in the repository.
COPY rust/plugin-bootstrap/grapl-plugin-bootstrap-init.service /
COPY rust/plugin-bootstrap/grapl-plugin.service /

##### node-identifier
FROM rust-dist AS node-identifier-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/node-identifier /
ENTRYPOINT ["/node-identifier"]

##### node-identifier-retry
FROM rust-dist AS node-identifier-retry-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/node-identifier-retry /
ENTRYPOINT ["/node-identifier-retry"]

##### sysmon-generator
FROM rust-dist AS sysmon-generator-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/sysmon-generator /
ENTRYPOINT ["/sysmon-generator"]

##### osquery-generator
FROM rust-dist AS osquery-generator-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/osquery-generator /
ENTRYPOINT ["/osquery-generator"]

##### web-ui
FROM rust-dist AS grapl-web-ui-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/grapl-web-ui /
COPY rust/grapl-web-ui/frontend /frontend
ENTRYPOINT ["/grapl-web-ui"]

##### model plugin deployer
FROM rust-dist AS model-plugin-deployer

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/model-plugin-deployer /
ENTRYPOINT ["/model-plugin-deployer"]

##### organization-management
FROM rust-dist AS organization-management-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/organization-management /
ENTRYPOINT ["/organization-management"]

##### pipeline-ingress
FROM rust-dist AS pipeline-ingress-deploy

# Hadolint v2.8.0 appears to be mistaken, this is a false positive
# https://github.com/hadolint/hadolint/wiki/DL3022
# hadolint ignore=DL3022
COPY --from=build /outputs/pipeline-ingress /
ENTRYPOINT ["/pipeline-ingress"]
