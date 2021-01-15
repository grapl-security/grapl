# valid values: s3, container, none
ARG SCCACHE_LOCATION=container

#
# base
#
FROM rust:1-slim-buster AS base

ARG PROFILE=debug
ARG TARGET=x86_64-unknown-linux-musl

RUN apt-get update && apt-get install -y --no-install-recommends \
        musl-dev \
        musl-tools \
        wait-for-it \
        wget \
        zip \
    && rm -rf /var/lib/apt/lists/*

RUN adduser \
        --disabled-password \
        --gecos '' \
        --uid 19999 \
        --home /home/grapl \
        --shell /bin/bash \
        grapl

USER grapl
ENV CARGO_HOME=/home/grapl/.cargo
WORKDIR /home/grapl

RUN mkdir /home/grapl/.cargo; \
    if test "${TARGET}" = "x86_64-unknown-linux-musl"; then \
      rustup target add x86_64-unknown-linux-musl; \
    fi

# copy sources
COPY --chown=grapl . .

#
# build using docker volume for cache
#
FROM base AS build-sccache-none

RUN if test "${PROFILE}" = "release"; then \
      cargo build --target=${TARGET} --release; \
    elif test "${PROFILE}" = "debug"; then \
      cargo build --target=${TARGET}; \
    else \
      echo "ERROR: Unknown build profile: ${PROFILE}"; \
      exit 1; \
    fi

#
# sccache base
#
FROM base AS sccache

ENV SCCACHE_DIR=/home/grapl/sccache
ENV RUSTC_WRAPPER=/home/grapl/bin/sccache

# Waiting on the following sccache PR to land to better support S3:
# https://github.com/mozilla/sccache/pull/869
RUN if test "${SCCACHE_LOCATION}" != "s3" ; then \
      cd /tmp && \
      wget -q https://github.com/mozilla/sccache/releases/download/0.2.14/sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
      tar xvzf sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
      mkdir -p /home/grapl/bin && \
      cp sccache-0.2.14-x86_64-unknown-linux-musl/sccache /home/grapl/bin/sccache; \
    fi

# RUN cd /tmp && \
#     wget -q https://github.com/mozilla/sccache/releases/download/0.2.14/sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
#     tar xvzf sccache-0.2.14-x86_64-unknown-linux-musl.tar.gz && \
#     mkdir /home/grapl/bin && \
#     cp sccache-0.2.14-x86_64-unknown-linux-musl/sccache /home/grapl/bin/sccache


# use docker volume cache for sccache
FROM sccache AS build-sccache-container

RUN --mount=type=cache,uid=19999,gid=19999,target=/home/grapl/.cargo/registry \
    --mount=type=cache,uid=19999,gid=19999,target=/home/grapl/sccache \
    if test "${PROFILE}" = "release"; then \
      cargo build --target=${TARGET} --release; \
    elif test "${PROFILE}" = "debug"; then \
      cargo build --target=${TARGET}; \
    else \
      echo "ERROR: Unknown build profile: ${PROFILE}"; \
      exit 1; \
    fi

# build-sccache-host is not currently supported
# FROM sccache AS build-sccache-host

# build using sccache from S3
FROM sccache AS build-sccache-s3

ARG SCCACHE_BUCKET

ENV SCCACHE_BUCKET=${SCCACHE_BUCKET}
ENV SCCACHE_S3_KEY_PREFIX=sccache

SHELL ["/bin/bash", "-c"]

RUN --mount=type=cache,uid=19999,gid=19999,target=/home/grapl/.cargo/registry \
    --mount=type=cache,uid=19999,gid=19999,target=/home/grapl/sccache \
    --mount=type=secret,uid=19999,gid=19999,id=aws,dst=/home/grapl/awscreds \
    source /home/grapl/awscreds && \
    if test "${PROFILE}" = "release"; then \
      cargo build --target=${TARGET} --release; \
    elif test "${PROFILE}" = "debug"; then \
      cargo build --target=${TARGET}; \
    else \
      echo "ERROR: Unknown build profile: ${PROFILE}"; \
      exit 1; \
    fi


# create stage alias for easy reference
FROM build-sccache-${SCCACHE_LOCATION} AS build

# zips
FROM build AS zips

ENV TAG=latest

SHELL ["/bin/bash", "-c"]

RUN mkdir /home/grapl/zips; \
    grapl-zip() { \
      TMPDIR=$(mktemp -d); \
      cd $TMPDIR; \
      cp "/home/grapl/target/${TARGET}/${PROFILE}/$1" bootstrap && \
      zip -q -9 -dg /home/grapl/zips/${f}.zip bootstrap; \
    }; \
    for f in analyzer-dispatcher \
			       metric-forwarder; \
    do \
        grapl-zip "$f" & \
    done; \
    wait

CMD for f in $(ls /home/grapl/zips | sed -e 's/.zip$//g'); do \
      cp -r /home/grapl/zips/${f}.zip ./${f}-${TAG}.zip; \
    done

#
# images for running services
#
FROM alpine AS rust-dist

ARG PROFILE=debug
ARG TARGET=x86_64-unknown-linux-musl

RUN apk add --no-cache libgcc

USER nobody

# analyzer-dispatcher
FROM rust-dist AS analyzer-dispatcher-deploy

COPY --from=build "/home/grapl/target/${TARGET}/${PROFILE}/analyzer-dispatcher" /analyzer-dispatcher
CMD /analyzer-dispatcher

# generic-subgraph-generator
FROM rust-dist AS generic-subgraph-generator-deploy

COPY --from=build "/home/grapl/target/${TARGET}/${PROFILE}/generic-subgraph-generator" /generic-subgraph-generator
CMD /generic-subgraph-generator

# graph-merger
FROM rust-dist AS graph-merger-deploy

COPY --from=build "/home/grapl/target/${TARGET}/${PROFILE}/graph-merger" /graph-merger
CMD /graph-merger

# metric-forwarder
FROM rust-dist AS metric-forwarder-deploy

COPY --from=build "/home/grapl/target/${TARGET}/${PROFILE}/metric-forwarder" /metric-forwarder
CMD /metric-forwarder

# node-identifier
FROM rust-dist AS node-identifier-deploy

COPY --from=build "/home/grapl/target/${TARGET}/${PROFILE}/node-identifier" /node-identifier
CMD /node-identifier

# node-identifier-retry-handler
FROM rust-dist AS node-identifier-retry-handler-deploy

COPY --from=build "/home/grapl/target/${TARGET}/${PROFILE}/node-identifier-retry-handler" /node-identifier-retry-handler
CMD /node-identifier-retry-handler

# sysmon-subgraph-generator
FROM rust-dist AS sysmon-subgraph-generator-deploy

COPY --from=build "/home/grapl/target/${TARGET}/${PROFILE}/sysmon-subgraph-generator" /sysmon-subgraph-generator
CMD /sysmon-subgraph-generator

# osquery-subgraph-generator
FROM rust-dist AS osquery-subgraph-generator-deploy

COPY --from=build "/home/grapl/target/${TARGET}/${PROFILE}/osquery-subgraph-generator" /osquery-subgraph-generator
CMD /osquery-subgraph-generator
