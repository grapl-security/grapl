FROM grapl/grapl-rust-base:$TAG as grapl-rust-build
ARG TAG=latest
ARG CARGO_PROFILE=debug
WORKDIR /tmp
RUN wget -q https://github.com/mozilla/sccache/releases/download/v0.2.15/sccache-v0.2.15-x86_64-unknown-linux-musl.tar.gz && \
    tar xvzf sccache-v0.2.15-x86_64-unknown-linux-musl.tar.gz && \
    chmod +x sccache-v0.2.15-x86_64-unknown-linux-musl/sccache && \
    cp sccache-v0.2.15-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache
ENV RUSTC_WRAPPER=/usr/local/bin/sccache
# Install rust toolchain before copying sources to avoid unecessarily
# resinstalling on source file changes.
WORKDIR /home/grapl
COPY rust/rust-toolchain rust/rust-toolchain
WORKDIR /home/grapl/rust
# 'rustup show' will install components in the rust-toolchain file
RUN rustup show
# copy sources
WORKDIR /home/grapl
COPY proto proto
COPY rust rust
WORKDIR /home/grapl/rust
RUN --mount=type=cache,mode=0777,target=/root/.cache/sccache \
    --mount=type=secret,id=rust_env,dst=/home/grapl/env \
    source /home/grapl/env; \
    case "${CARGO_PROFILE}" in \
      debug) \
        cargo build ;; \
      release) \
        cargo build --release ;; \
      *) \
        echo "ERROR: Unknown profile: ${CARGO_PROFILE}"; \
        exit 1 ;; \
    esac

# metric-forwarder-zip
################################################################################
FROM grapl-rust-build AS metric-forwarder-zip
RUN mkdir -p /home/grapl/zips; \
    grapl-zip() { \
      TMPDIR="$(mktemp -d)"; \
      cd "$TMPDIR"; \
      cp "/home/grapl/rust/target/${CARGO_PROFILE}/${1}" bootstrap && \
      zip --quiet -9 "/home/grapl/zips/${1}.zip" bootstrap; \
    }; \
    grapl-zip metric-forwarder

# build test stages
################################################################################
FROM grapl-rust-build AS build-test-unit
RUN --mount=type=cache,mode=0777,target=/root/.cache/sccache \
    --mount=type=secret,id=rust_env,dst=/home/grapl/env \
    source /grapl/env; \
    cargo test --no-run

FROM grapl-rust-build AS build-test-integration
RUN --mount=type=cache,mode=0777,target=/root/.cache/sccache \
    --mount=type=secret,id=rust_env,dst=/home/grapl/env \
    source /home/grapl/env; \
    cargo test --features node-identifier/integration,sqs-executor/integration --test '*' --no-run

# images for running services
################################################################################
FROM grapl/grapl-base:$TAG AS rust-dist
ARG TAG=latest
ARG CARGO_PROFILE=debug

# analyzer-dispatcher
FROM rust-dist AS analyzer-dispatcher
COPY --from=build "/home/grapl/rust/target/${CARGO_PROFILE}/analyzer-dispatcher" /home/grapl/analyzer-dispatcher
CMD ["/home/grapl/analyzer-dispatcher"]

# generic-subgraph-generator
FROM rust-dist AS generic-subgraph-generator
COPY --from=build "/home/grapl/rust/target/${CARGO_PROFILE}/generic-subgraph-generator" /home/grapl/generic-subgraph-generator
CMD ["/home/grapl/generic-subgraph-generator"]

# graph-merger
FROM rust-dist AS graph-merger
COPY --from=build "/home/grapl/rust/target/${CARGO_PROFILE}/graph-merger" /home/grapl/graph-merger
CMD ["/home/grapl/graph-merger"]

# node-identifier
FROM rust-dist AS node-identifier
COPY --from=build "/home/grapl/rust/target/${CARGO_PROFILE}/node-identifier" /home/grapl/node-identifier
CMD ["/home/grapl/node-identifier"]

# node-identifier-retry-handler
FROM rust-dist AS node-identifier-retry-handler
COPY --from=build "/home/grapl/rust/target/${CARGO_PROFILE}/node-identifier-retry-handler" /home/grapl/node-identifier-retry-handler
CMD ["/home/grapl/node-identifier-retry-handler"]

# sysmon-subgraph-generator
FROM rust-dist AS sysmon-subgraph-generator
COPY --from=build "/home/grapl/rust/target/${CARGO_PROFILE}/sysmon-subgraph-generator" /home/grapl/sysmon-subgraph-generator
CMD ["/home/grapl/sysmon-subgraph-generator"]

# osquery-subgraph-generator
FROM rust-dist AS osquery-subgraph-generator
COPY --from=build "/home/grapl/rust/target/${CARGO_PROFILE}/osquery-subgraph-generator" /home/grapl/osquery-subgraph-generator
CMD ["/home/grapl/osquery-subgraph-generator"]
