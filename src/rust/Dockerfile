# syntax=docker/dockerfile:1.4
# We use the above syntax for here documents:
# https://github.com/moby/buildkit/blob/master/frontend/dockerfile/docs/syntax.md#user-content-here-documents

ARG RUST_VERSION

FROM rust:${RUST_VERSION}-slim-bullseye AS base

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
        curl=7.74.0-1.3+deb11u1 \
        jq=1.6-2.1 \
        unzip=6.0-26 \
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

ENV RUST_INTEGRATION_TEST_FEATURES="generator-dispatcher/integration_tests,graph-merger/integration_tests,node-identifier/integration_tests,pipeline-ingress/integration_tests,plugin-registry/integration_tests,sysmon-generator/integration_tests,organization-management/integration_tests"
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
        jq -r "select((.profile.test == true) and (.features[] | contains(\"integration_tests\"))) | .filenames[]" | \
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
COPY --from=build /nomad/nomad /bin

# Grab the example generator so we can deploy it in test_deploy_plugin
RUN mkdir -p /test-fixtures
COPY --from=build /outputs/example-generator /test-fixtures
COPY --from=build /outputs/sysmon-generator /test-fixtures

COPY --from=build-test-integration /grapl/tests /tests
# Named context support https://github.com/hadolint/hadolint/issues/830
# hadolint ignore=DL3022
COPY --from=test-ctx ./run/rust-integration-tests.sh /
ENTRYPOINT [ "/rust-integration-tests.sh" ]


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

##### graph-merger
FROM rust-dist AS graph-merger-deploy

COPY --from=build /outputs/graph-merger /
ENTRYPOINT ["/graph-merger"]

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

##### sysmon-generator
FROM rust-dist AS sysmon-generator-deploy

COPY --from=build /outputs/sysmon-generator-kafka-legacy /
ENTRYPOINT ["/sysmon-generator-kafka-legacy"]

##### generator-executor
FROM rust-dist AS generator-executor-deploy

COPY --from=build /outputs/generator-executor /
ENTRYPOINT ["/generator-executor"]

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

##### uid-allocator
FROM rust-dist AS uid-allocator-deploy

COPY --from=build /outputs/uid-allocator /
ENTRYPOINT ["/uid-allocator"]

##### generator-dispatcher
FROM rust-dist AS generator-dispatcher-deploy

COPY --from=build /outputs/generator-dispatcher /
ENTRYPOINT ["/generator-dispatcher"]

##### event-source
FROM rust-dist as event-source-deploy
COPY --from=build /outputs/event-source /
ENTRYPOINT ["/event-source"]
