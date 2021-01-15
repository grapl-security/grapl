# valid values: s3, container, none
ARG SCCACHE_LOCATION=container

#
# base
#
FROM rust:1-slim-buster AS base

ARG PROFILE=debug

RUN apt-get update && apt-get install -y --no-install-recommends \
        wait-for-it \
        wget \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /grapl

# copy sources
COPY . .

#
# build using docker volume for cache
#
FROM base AS build-sccache-none

RUN set -eux; \
    case "${PROFILE}" in \
      debug) \
        cargo build; \
        cargo test --no-run ;; \
      release) \
        cargo build --release ;; \
      *) \
        echo "ERROR: Unknown profile: ${PROFILE}"; \
        exit 1 ;; \
    esac

#
# sccache base
#
FROM base AS sccache

ENV SCCACHE_DIR=/grapl/sccache
ENV RUSTC_WRAPPER=/grapl/bin/sccache

# Waiting on the following sccache PR to land to better support S3:
# https://github.com/mozilla/sccache/pull/869
RUN if test "${SCCACHE_LOCATION}" != "s3" ; then \
      cd /tmp && \
      wget -q https://github.com/mozilla/sccache/releases/download/0.2.14/sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
      tar xvzf sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
      mkdir -p /grapl/bin && \
      cp sccache-0.2.14-x86_64-unknown-linux-musl/sccache /grapl/bin/sccache; \
    fi

# RUN cd /tmp && \
#     wget -q https://github.com/mozilla/sccache/releases/download/0.2.14/sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
#     tar xvzf sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
#     mkdir /home/grapl/bin && \
#     cp sccache-0.2.14-x86_64-unknown-linux-musl/sccache /home/grapl/bin/sccache


# use docker volume cache for sccache
FROM sccache AS build-sccache-container

RUN --mount=type=cache,mode=0777,target=/usr/local/cargo/registry \
    --mount=type=cache,mode=0777,target=/grapl/sccache \
    set -eux; \
    case "${PROFILE}" in \
      debug) \
        cargo build; \
        cargo test --no-run ;; \
      release) \
        cargo build --release ;; \
      *) \
        echo "ERROR: Unknown profile: ${PROFILE}" \
        exit 1 ;; \
    esac

# build-sccache-host is not currently supported
# FROM sccache AS build-sccache-host

# build using sccache from S3
FROM sccache AS build-sccache-s3

ARG SCCACHE_BUCKET

ENV SCCACHE_BUCKET=${SCCACHE_BUCKET}
ENV SCCACHE_S3_KEY_PREFIX=sccache

SHELL ["/bin/bash", "-c"]

RUN --mount=type=cache,mode=0777,target=/usr/local/cargo/registry \
    --mount=type=cache,mode=0777,target=/grapl/sccache \
    --mount=type=secret,id=aws,dst=/grapl/awscreds \
    source /grapl/awscreds; \
    set -eux; \
    case "${PROFILE}" in \
      debug) \
        cargo build; \
        cargo test --no-run ;; \
      release) \
        cargo build --release ;; \
      *) \
        echo "ERROR: Unknown profile: ${PROFILE}"; \
        exit 1 ;; \
    esac


# create stage alias for easy reference
FROM build-sccache-${SCCACHE_LOCATION} AS build


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
