FROM rust:1-slim-buster AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-dev \
    musl-tools \
    wait-for-it \
    wget \
    netcat \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl


# sccache
FROM base AS sccache

RUN cd /tmp && \
    wget -q https://github.com/mozilla/sccache/releases/download/0.2.14/sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
    tar xvzf sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
    cp sccache-0.2.14-x86_64-unknown-linux-musl/sccache /usr/bin/

ENV SCCACHE_DIR=/sccache
ENV RUSTC_WRAPPER=sccache

#
# images for running locally
#

# analyzer-dispatcher
FROM alpine AS analyzer-dispatcher-deploy

USER nobody

ARG TARGET=debug
COPY "${TARGET}"/analyzer-dispatcher /analyzer-dispatcher
CMD /analyzer-dispatcher

# generic-subgraph-generator
FROM alpine AS generic-subgraph-generator-deploy

USER nobody

ARG TARGET=debug
COPY "${TARGET}"/generic-subgraph-generator /generic-subgraph-generator
CMD /generic-subgraph-generator

# graph-merger
FROM alpine AS graph-merger-deploy

USER nobody

ARG TARGET=debug
COPY "${TARGET}"/graph-merger /graph-merger
CMD /graph-merger

# metric-forwarder
FROM alpine AS metric-forwarder-deploy

USER nobody

ARG TARGET=debug
COPY "${TARGET}"/metric-forwarder /metric-forwarder
CMD /metric-forwarder

# node-identifier
FROM alpine AS node-identifier-deploy

USER nobody

ARG TARGET=debug
COPY "${TARGET}"/node-identifier /node-identifier
CMD /node-identifier

# node-identifier-retry-handler
FROM alpine AS node-identifier-retry-handler-deploy

USER nobody

ARG TARGET=debug
COPY "${TARGET}"/node-identifier-retry-handler /node-identifier-retry-handler
CMD /node-identifier-retry-handler

# sysmon-subgraph-generator
FROM alpine AS sysmon-subgraph-generator-deploy

USER nobody

ARG TARGET=debug
COPY "${TARGET}"/sysmon-subgraph-generator /sysmon-subgraph-generator
CMD /sysmon-subgraph-generator

# osquery-subgraph-generator
FROM alpine AS osquery-subgraph-generator-deploy

USER nobody

ARG TARGET=debug
COPY "${TARGET}"/osquery-subgraph-generator /osquery-subgraph-generator
CMD /osquery-subgraph-generator