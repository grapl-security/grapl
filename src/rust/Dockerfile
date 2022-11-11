# syntax=docker/dockerfile:1.4
# We use the above syntax for here documents:
# https://github.com/moby/buildkit/blob/master/frontend/dockerfile/docs/syntax.md#user-content-here-documents

ARG RUST_VERSION

FROM rust:${RUST_VERSION}-slim-bullseye AS base

ARG PROTOC_VERSION
ARG RUST_BUILD=dev-local-grapl

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
        curl=7.74.0-1.3+deb11u3 \
        jq=1.6-2.1 \
        unzip=6.0-26+deb11u1 \
    && apt-get install --yes --no-install-recommends \
        build-essential=12.9 \
        cmake=3.18.4-2+deb11u1 \
        libssl-dev=1.1.1n-0+deb11u3 \
        perl=5.32.1-4+deb11u2 \
        pkg-config=0.29.2-1 \
        tcl=8.6.11+1

# Grab a Nomad binary, which we use for parsing HCL2-with-variables into JSON:
# - in plugin-registry integration tests
# - in plugin-registry image
WORKDIR /nomad
RUN <<EOF
NOMAD_VERSION="1.2.4"  # TODO: ARG-ify this like PROTOC_VERSION
ZIP_NAME="nomad_${NOMAD_VERSION}_linux_amd64.zip"
curl --remote-name --proto '=https' --tlsv1.2 -sSf \
  "https://releases.hashicorp.com/nomad/${NOMAD_VERSION}/${ZIP_NAME}"
unzip "${ZIP_NAME}"
rm "${ZIP_NAME}"
EOF

WORKDIR /tmp
RUN <<EOF
    echo "${PROTOC_VERSION}"
    PB_REL="https://github.com/protocolbuffers/protobuf/releases"
    ZIP_PATH="/tmp/protoc.zip"

    # Download the zip
    curl \
        --proto '=https' --tlsv1.2 -sSf \
        --location \
        --output "${ZIP_PATH}" \
        "${PB_REL}/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip"

    mkdir --parents ~/.local
    # -d: Unzip it into / - which drops `protoc` in /bin.
    unzip -d / "${ZIP_PATH}"
    rm "${ZIP_PATH}"
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
        cargo build;;
      dev-local-grapl)
        cargo build --profile dev-local-grapl;;
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
        libssl-dev=1.1.1n-0+deb11u3 \
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

# You can override which integration tests are run by
# specifying this ARG as a comma-separated value. See docker-bake.hcl for more info.
ARG INTEGRATION_TEST_FEATURES_OVERRIDE

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

# hadolint ignore=SC3054
RUN <<EOF
INTEGRATION_TEST_FEATURES=(
analyzer-dispatcher/integration_tests
e2e-tests/integration_tests
generator-dispatcher/integration_tests
graph-merger/integration_tests
graph-query/integration_tests
graph-schema-manager/integration_tests
grapl-metrics/integration_tests
grapl-web-ui/integration_tests
node-identifier/integration_tests
organization-management/integration_tests
pipeline-ingress/integration_tests
plugin-registry/integration_tests
plugin-work-queue/integration_tests
sysmon-generator/integration_tests
uid-allocator/integration_tests
)

# cargo --features expects a comma-separated feature1,feature2,feature3
function join_by { local IFS="$1"; shift; echo "$*"; }

if [ -n "${INTEGRATION_TEST_FEATURES_OVERRIDE}" ]; then
  # Override list of features with INTEGRATION_TEST_FEATURES_OVERRIDE if it is set
  echo "${INTEGRATION_TEST_FEATURES_OVERRIDE}" > integration_test_features.csv
else
  join_by , "${INTEGRATION_TEST_FEATURES[@]}" > integration_test_features.csv
fi

EOF

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
<<EOF
    # First, run cargo test without --message-format=json; this will expose
    # errors to the developer over stdout/err in human-readable format.
    cargo test \
        --features "$(cat integration_test_features.csv)" \
        --no-run \
        --test "*"

    # Run cargo test again - everything's already compiled - but this time,
    # the message is used as input to the jq script
    cargo test \
        --features "$(cat integration_test_features.csv)" \
        --no-run \
        --message-format=json \
        --test "*" | \
        jq -r "select((.profile.test == true) and (.features[] | contains(\"integration_tests\"))) | .filenames[]" | \
        xargs \
          --replace="{}" \
          cp "{}" "${TEST_DIR}/"
EOF


# integration tests distribution
################################################################################
# We're unable to use one of the 'distroless' container images as a base here
# because our integration tests require zlib shared library, but we don't have
# a way of including that in the base image. With a debian image we can apt
# install as needed, but the debian image we're using has zlib already.
FROM debian:bullseye-slim AS integration-tests

# Apt package versioning may be helpful elsewhere, like the production images,
# but not here. Pinning versions becomes an issue when versions get pulled.
# hadolint ignore=DL3008
RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rust-tests-apt \
    apt-get update \
    && apt-get install --yes --no-install-recommends \
        ca-certificates \
        parallel

# Put the Nomad binary on PATH
# so that it's available to integration test consumers of NomadCli
COPY --from=build /nomad/nomad /bin
RUN mkdir -p /test-fixtures

# Copy in the Example Schemas, which we use in Schema Manager integration tests
# hadolint-ignore reason: https://github.com/hadolint/hadolint/issues/830
# hadolint ignore=DL3022
COPY --from=etc-ctx example_schemas /test-fixtures/example_schemas

# Copy in the 36-line sysmon event-log, which we use in E2E tests.
# hadolint-ignore reason: https://github.com/hadolint/hadolint/issues/830
# hadolint ignore=DL3022
COPY --from=etc-ctx sample_data/36_eventlog.xml /test-fixtures

# Copy in an example Analyzers, for integration testing.
# hadolint-ignore reason: https://github.com/hadolint/hadolint/issues/830
# hadolint ignore=DL3022
COPY --from=dist-ctx suspicious_svchost_analyzer.pex /test-fixtures
# hadolint ignore=DL3022
COPY --from=dist-ctx process_named_svchost_analyzer.pex /test-fixtures

# Copy in the example generator so we can deploy it in test_deploy_plugin
COPY --from=build /outputs/example-generator /test-fixtures
COPY --from=build /outputs/sysmon-generator /test-fixtures

COPY --from=build-test-integration /grapl/tests /tests

# We need to wait for Nomad/Consul service discovery sidecar to finish setting
# up before some of our tests attempt to connect to local sidecars. Without
# doing so there is a race between between the sidecar and tests trying to
# connect. Without waiting we can reliably get connection refuse errors:
#
#   error trying to connect: tcp connect error: Connection refused (os error
#   111)
#
# Although the gRPC Client Executor is equipped with retry logic, it is not the
# only network client being used in these tests, nor should it be assumed to be
# the only. Grapl web, for example, would not use that to test all its APIs.
#
# While the web tests at this point could connect directly to the web-ui
# service at http://web-ui.service.dc1.consul:1234 without this annoying wait,
# doing so won't work for authenticated APIs because the web-ui service sets
# the session cookie as Secure-Only, which means it won't be sent to the host,
# and sending a Secure-Only cookie over a non-TLS connection only works when
# the endpoint address is 127.0.0.1. When/if the Nomad ingres service (not to
# be confused with the Grapl Pipeline ingress service) serves the web-ui behind
# TLS then we can skip this wait and connect right away via
# https://web-ui.service.dc1.consul:1234. This would have the additional
# benefit of including the Nomad ingress-service in this integration test.
#
# TODO: remove this when we can either a) rely on local service discovery to
# finish setting up before we run our tests, or b) connect via the Nomad
# ingress-service via HTTPS.
#
# Note: do not expand the following '-v' to '--verbose', the effect is
# different and not desired.
ENTRYPOINT [ "sh", "-c", "sleep 10; find /tests -type f | parallel -v '{}' '2>&1'" ]


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

##### export-rust-build-artifacts-to-dist
# There are a number of artifacts we want to bring back to the host OS.
# This image will eventually dump its root contents into the host's `dist/`
# courtesy of its `docker-bake.hcl` specification.
FROM scratch AS export-rust-build-artifacts-to-dist

COPY --from=build /outputs/plugin-bootstrap-init /plugin-bootstrap-init/
COPY --from=build /outputs/example-generator /
# Just to clarify: we're copying these .service files from the repository,
# through Docker, and then back out to the dist directory in the repository.
COPY rust/plugin-bootstrap/grapl-plugin-bootstrap-init.service /plugin-bootstrap-init/
COPY rust/plugin-bootstrap/grapl-plugin.service /plugin-bootstrap-init/

##### analyzer-execution-sidecar
FROM rust-dist AS analyzer-execution-sidecar-deploy

COPY --from=build /outputs/analyzer-execution-sidecar /
ENTRYPOINT ["/analyzer-execution-sidecar"]

##### graph-merger
FROM rust-dist AS graph-merger-deploy

COPY --from=build /outputs/graph-merger /
ENTRYPOINT ["/graph-merger"]

##### graph-query
FROM rust-dist AS graph-query-deploy

COPY --from=build /outputs/graph-query /
ENTRYPOINT ["/graph-query"]

##### graph-mutation
FROM rust-dist AS graph-mutation-deploy

COPY --from=build /outputs/graph-mutation /
ENTRYPOINT ["/graph-mutation"]

##### graph-query-proxy
FROM rust-dist as graph-query-proxy-deploy

COPY --from=build /outputs/graph-query-proxy /
ENTRYPOINT ["/graph-query-proxy"]

##### plugin-work-queue
FROM rust-dist AS plugin-work-queue-deploy

COPY --from=build /outputs/plugin-work-queue /
ENTRYPOINT ["/plugin-work-queue"]

##### plugin-registry
FROM rust-dist AS plugin-registry-deploy

COPY --from=build /outputs/plugin-registry /
# Put the Nomad binary on PATH for NomadCli class
COPY --from=build /nomad/nomad /bin
ENTRYPOINT ["/plugin-registry"]

##### plugin-bootstrap
FROM rust-dist AS plugin-bootstrap-deploy

COPY --from=build /outputs/plugin-bootstrap /
ENTRYPOINT ["/plugin-bootstrap"]

##### node-identifier
FROM rust-dist AS node-identifier-deploy

COPY --from=build /outputs/node-identifier /
ENTRYPOINT ["/node-identifier"]

##### generator-execution-sidecar
FROM rust-dist AS generator-execution-sidecar-deploy

COPY --from=build /outputs/generator-execution-sidecar /
ENTRYPOINT ["/generator-execution-sidecar"]

##### web-ui
FROM rust-dist AS grapl-web-ui-deploy

COPY --from=build /outputs/grapl-web-ui /
# Named context support https://github.com/hadolint/hadolint/issues/830
# hadolint ignore=DL3022
COPY --from=dist-ctx frontend /frontend
ENTRYPOINT ["/grapl-web-ui"]

##### organization-management
FROM rust-dist AS organization-management-deploy

COPY --from=build /outputs/organization-management /
ENTRYPOINT ["/organization-management"]

##### pipeline-ingress
FROM rust-dist AS pipeline-ingress-deploy

COPY --from=build /outputs/pipeline-ingress /
ENTRYPOINT ["/pipeline-ingress"]

##### scylla-provisioner
FROM rust-dist AS scylla-provisioner-deploy

COPY --from=build /outputs/scylla-provisioner /
ENTRYPOINT ["/scylla-provisioner"]

##### uid-allocator
FROM rust-dist AS uid-allocator-deploy

COPY --from=build /outputs/uid-allocator /
ENTRYPOINT ["/uid-allocator"]

##### analyzer-dispatcher
FROM rust-dist AS analyzer-dispatcher-deploy

COPY --from=build /outputs/analyzer-dispatcher /
ENTRYPOINT ["/analyzer-dispatcher"]

##### generator-dispatcher
FROM rust-dist AS generator-dispatcher-deploy

COPY --from=build /outputs/generator-dispatcher /
ENTRYPOINT ["/generator-dispatcher"]

##### event-source
FROM rust-dist as event-source-deploy
COPY --from=build /outputs/event-source /
ENTRYPOINT ["/event-source"]

##### kafka-retry
FROM rust-dist as kafka-retry-deploy
COPY --from=build /outputs/kafka-retry /
ENTRYPOINT ["/kafka-retry"]

##### graph-schema-manager
FROM rust-dist AS graph-schema-manager-deploy

COPY --from=build /outputs/graph-schema-manager /
ENTRYPOINT ["/graph-schema-manager"]
