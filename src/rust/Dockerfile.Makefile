FROM rust:1-slim-buster AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-dev \
    musl-tools \
    wait-for-it \
    wget \
    zip \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl


# sccache
FROM base AS sccache

RUN cd /tmp && \
    wget -q https://github.com/mozilla/sccache/releases/download/0.2.14/sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
    tar xvzf sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
    cp sccache-0.2.14-x86_64-unknown-linux-musl/sccache /usr/bin/


# build sources 
# create separate stage for those who want prior stage independently
FROM sccache AS build-sccache

ARG TARGET=debug

ENV HOME=/rust
ENV SCCACHE_DIR=$HOME/sccache
ENV RUSTC_WRAPPER=/usr/bin/sccache

WORKDIR $HOME

COPY . .

RUN --mount=type=cache,target=/rust/sccache \
    --mount=type=cache,target=/usr/local/cargo/registry \
    if test "${TARGET}" = "release"; then \
      cargo build --release; \
      sccache -s; \
    elif test "${TARGET}" = "debug"; then \
      cargo build; \
      sccache -s; \
    else \
      echo "ERROR: Unknown build target: ${TARGET}"; \
      exit 1; \
    fi


# zips
FROM build-sccache AS zips

ENV TAG=latest

SHELL ["/bin/bash", "-c"]

RUN mkdir /zips; \
    grapl-zip() { \
      TMPDIR=$(mktemp -d); \
      cd $TMPDIR; \
      cp "${HOME}/target/x86_64-unknown-linux-musl/${TARGET}/$1" bootstrap && \
      zip -q -dg /zips/${f}-${TAG}.zip bootstrap; \
    }; \
    for f in analyzer-dispatcher \
			       generic-subgraph-generator \
			       graph-merger \
			       metric-forwarder \
			       node-identifier \
			       node-identifier-retry-handler \
			       sysmon-subgraph-generator \
			       osquery-subgraph-generator; \
    do \
        grapl-zip "$f" & \
    done; \
    wait

CMD cp -r /zips/. .


#
# images for running services
#
FROM alpine AS rust-dist

RUN apk add --no-cache libgcc

USER nobody

# analyzer-dispatcher
FROM rust-dist AS analyzer-dispatcher-deploy

ARG TARGET=debug
COPY --from=build-sccache /rust/target/x86_64-unknown-linux-musl/${TARGET}/analyzer-dispatcher /analyzer-dispatcher
CMD /analyzer-dispatcher

# generic-subgraph-generator
FROM rust-dist AS generic-subgraph-generator-deploy

ARG TARGET=debug
COPY --from=build-sccache /rust/target/x86_64-unknown-linux-musl/${TARGET}/generic-subgraph-generator /generic-subgraph-generator
CMD /generic-subgraph-generator

# graph-merger
FROM rust-dist AS graph-merger-deploy

ARG TARGET=debug
COPY --from=build-sccache /rust/target/x86_64-unknown-linux-musl/${TARGET}/graph-merger /graph-merger
CMD /graph-merger

# metric-forwarder
FROM rust-dist AS metric-forwarder-deploy

ARG TARGET=debug
COPY --from=build-sccache /rust/target/x86_64-unknown-linux-musl/${TARGET}/metric-forwarder /metric-forwarder
CMD /metric-forwarder

# node-identifier
FROM rust-dist AS node-identifier-deploy

ARG TARGET=debug
COPY --from=build-sccache /rust/target/x86_64-unknown-linux-musl/${TARGET}/node-identifier /node-identifier
CMD /node-identifier

# node-identifier-retry-handler
FROM rust-dist AS node-identifier-retry-handler-deploy

ARG TARGET=debug
COPY --from=build-sccache /rust/target/x86_64-unknown-linux-musl/${TARGET}/node-identifier-retry-handler /node-identifier-retry-handler
CMD /node-identifier-retry-handler

# sysmon-subgraph-generator
FROM rust-dist AS sysmon-subgraph-generator-deploy

ARG TARGET=debug
COPY --from=build-sccache /rust/target/x86_64-unknown-linux-musl/${TARGET}/sysmon-subgraph-generator /sysmon-subgraph-generator
CMD /sysmon-subgraph-generator

# osquery-subgraph-generator
FROM rust-dist AS osquery-subgraph-generator-deploy

ARG TARGET=debug
COPY --from=build-sccache /rust/target/x86_64-unknown-linux-musl/${TARGET}/osquery-subgraph-generator /osquery-subgraph-generator
CMD /osquery-subgraph-generator