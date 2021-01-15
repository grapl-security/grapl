FROM rust:1-slim-buster AS base

ARG PROFILE=debug
ARG GRAPL_RUSTC_WRAPPER

RUN apt-get update && apt-get install -y --no-install-recommends \
        wait-for-it \
        wget \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTC_WRAPPER=${GRAPL_RUSTC_WRAPPER}

SHELL ["/bin/bash", "-c"]

# Waiting on the following sccache PR to land to better support S3:
# https://github.com/mozilla/sccache/pull/869
# RUN cd /tmp && \
#     wget -q https://github.com/mozilla/sccache/releases/download/0.2.14/sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
#     tar xvzf sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
#     mkdir /home/grapl/bin && \
#     cp sccache-0.2.14-x86_64-unknown-linux-musl/sccache /home/grapl/bin/sccache

WORKDIR /grapl

# copy sources
COPY . .


#
# build
#
FROM base AS build

WORKDIR /grapl

RUN --mount=type=cache,mode=0777,target=/root/.cache/sccache \
    --mount=type=secret,id=aws,dst=/grapl/awscreds \
    source /grapl/awscreds; \
    case "${PROFILE}" in \
      debug) \
        cargo build ;; \
      release) \
        cargo build --release ;; \
      *) \
        echo "ERROR: Unknown profile: ${PROFILE}"; \
        exit 1 ;; \
    esac


#
# build test targets
#
FROM build AS build-test-unit

RUN --mount=type=cache,mode=0777,target=/root/.cache/sccache \
    --mount=type=secret,id=aws,dst=/grapl/awscreds \
    source /grapl/awscreds; \
    cargo test --no-run


FROM build AS build-test-integration

RUN --mount=type=cache,mode=0777,target=/root/.cache/sccache \
    --mount=type=secret,id=aws,dst=/grapl/awscreds \
    source /grapl/awscreds; \
    cargo test --manifest-path node-identifier/Cargo.toml --features integration --no-run


#
# images for running services
#
FROM debian:buster-slim AS rust-dist

ARG PROFILE=debug

USER nobody

# analyzer-dispatcher
FROM rust-dist AS analyzer-dispatcher-deploy

COPY --from=build "/grapl/target/${PROFILE}/analyzer-dispatcher" /
CMD ["/analyzer-dispatcher"]

# generic-subgraph-generator
FROM rust-dist AS generic-subgraph-generator-deploy

COPY --from=build "/grapl/target/${PROFILE}/generic-subgraph-generator" /
CMD ["/generic-subgraph-generator"]

# graph-merger
FROM rust-dist AS graph-merger-deploy

COPY --from=build "/grapl/target/${PROFILE}/graph-merger" /
CMD ["/graph-merger"]

# metric-forwarder
FROM rust-dist AS metric-forwarder-deploy

COPY --from=build "/grapl/target/${PROFILE}/metric-forwarder" /
CMD ["/metric-forwarder"]

# node-identifier
FROM rust-dist AS node-identifier-deploy

COPY --from=build "/grapl/target/${PROFILE}/node-identifier" /
CMD ["/node-identifier"]

# node-identifier-retry-handler
FROM rust-dist AS node-identifier-retry-handler-deploy

COPY --from=build "/grapl/target/${PROFILE}/node-identifier-retry-handler" /
CMD ["/node-identifier-retry-handler"]

# sysmon-subgraph-generator
FROM rust-dist AS sysmon-subgraph-generator-deploy

COPY --from=build "/grapl/target/${PROFILE}/sysmon-subgraph-generator" /
CMD ["/sysmon-subgraph-generator"]

# osquery-subgraph-generator
FROM rust-dist AS osquery-subgraph-generator-deploy

COPY --from=build "/grapl/target/${PROFILE}/osquery-subgraph-generator" /
CMD ["/osquery-subgraph-generator"]
