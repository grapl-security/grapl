# syntax=docker/dockerfile:1.3-labs

ARG RUST_VERSION

FROM rust:${RUST_VERSION}-slim-bullseye

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rust-build-env-apt <<EOF
    apt-get update
    # `libssl-dev` and `pkg-config` are needed for the initial
    # compilation of the `cargo-tarpaulin` tool itself.
    # build-essential, cmake, libssl-dev, perl, pkg-config, and tcl are needed
    # for building rust-rdkafka.
    apt-get install --yes --no-install-recommends \
         build-essential=12.9 \
         cmake=3.18.4-2+deb11u1 \
         libssl-dev=1.1.1k-1+deb11u2 \
         perl=5.32.1-4+deb11u2 \
         pkg-config=0.29.2-1 \
         tcl=8.6.11+1
EOF

# This is where tarpaulin gets installed; using a volume means we get
# to install it (and compile it!) once and forget it.
ENV CARGO_HOME=/grapl/cargo
# This is where all the compiled assets get dumped; using a volume
# allows speedier compiles because we don't have to start from scratch
# each time.
ENV CARGO_TARGET_DIR=/grapl/target
# Using a volume for this means we don't have to synchronize toolchain
# components every time we run the container.
ENV RUSTUP_HOME=/grapl/rustup

RUN mkdir --parents \
    "${CARGO_HOME}" \
    "${CARGO_TARGET_DIR}" \
    "${RUSTUP_HOME}" \
    && chmod --recursive 777 \
    "${CARGO_HOME}" \
    "${CARGO_TARGET_DIR}" \
    "${RUSTUP_HOME}"

VOLUME "${CARGO_TARGET_DIR}"
VOLUME "${CARGO_HOME}"
VOLUME "${RUSTUP_HOME}"
