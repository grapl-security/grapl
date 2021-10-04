FROM rust:1-slim-buster

# Necessary for rdkafka; pkg-config & libssl-dev for cargo-tarpaulin build
RUN --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update && apt-get install -y --no-install-recommends \
        zlib1g-dev \
        build-essential \
        pkg-config \
        libssl-dev \
        jq

SHELL ["/bin/bash", "-c"]

ENV CARGO_TARGET_DIR=/grapl/target
ENV CARGO_HOME=/grapl/cargo
ENV RUSTUP_HOME=/grapl/rustup

# Manually create Docker volume mount points so we can set the mode
# to make them a+w.
RUN mkdir --mode=777 --parents /grapl/{target,cargo,rustup}
