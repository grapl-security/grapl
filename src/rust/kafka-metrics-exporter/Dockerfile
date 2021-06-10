FROM rust:1-slim-buster AS base

ARG RUST_BUILD

SHELL ["/bin/bash", "-c"]

# Install rust toolchain before copying sources to avoid unecessarily
# resinstalling on source file changes.
WORKDIR /project
COPY rust-toolchain.toml rust-toolchain.toml
# 'rustup show' will install components in the rust-toolchain file
RUN rustup show

# copy sources
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY build.rs build.rs
copy proto/ proto/
COPY src/ src/
COPY tests/ tests/

# build-test
################################################################################
FROM base AS build-test

RUN --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update && apt-get install -y --no-install-recommends \
	python3 \
	cmake \
	libzstd-dev \
	build-essential \
	zlib1g-dev

RUN --mount=type=cache,target=/project/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry \
    case "${RUST_BUILD}" in \
      test) \
        cargo test --no-run ;; \
      test-integration) \
        cargo test --features integration --test "*" --no-run ;; \
      *) \
        echo "ERROR: Unknown RUST_BUILD option: ${RUST_BUILD}"; \
        exit 1 ;; \
    esac

# Save the mount cache to the Docker image, as these files are needed for
# running the integration tests in containers. 
RUN --mount=type=cache,target=/project/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cp -a /project/target /tmp/target && \
    cp -a /usr/local/cargo/registry /tmp/registry

RUN mv /tmp/target /project/target && \
    mv /tmp/registry /usr/local/cargo/registry
