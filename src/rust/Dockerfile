#
# Base image
#

FROM rust:1-slim-buster AS grapl-rust-base
RUN apt-get update && apt-get install -y --no-install-recommends \
    apt-utils \
    musl \
    musl-dev \
    musl-tools \
    protobuf-compiler \
    libzstd-dev \
    zip

ENV PROTOC /usr/bin/protoc
ENV PROTOC_INCLUDE /usr/include

ARG USERID="1000:1000"
USER $USERID

WORKDIR /grapl

RUN rustup target add x86_64-unknown-linux-musl

#
# Build image
#

FROM grapl-rust-base as grapl-rust-build

ARG release_target="debug"
ARG USERID="1000:1000"
USER $USERID

COPY analyzer-dispatcher analyzer-dispatcher
COPY derive-dynamic-node derive-dynamic-node
COPY generic-subgraph-generator generic-subgraph-generator
COPY graph-descriptions graph-descriptions
COPY graph-generator-lib graph-generator-lib
COPY graph-merger graph-merger
COPY grapl-config grapl-config
COPY grapl-observe grapl-observe
COPY metric-forwarder metric-forwarder
COPY node-identifier node-identifier
COPY sysmon-subgraph-generator sysmon-subgraph-generator
COPY Cargo.lock .
COPY Cargo.toml .

# Now do a clean build
RUN if test "${release_target}" = "release"; then \
      cargo build --target=x86_64-unknown-linux-musl --release; \
    elif test "${release_target}" = "debug"; then \
      cargo build --target=x86_64-unknown-linux-musl; \
    else \
      echo "ERROR: Unknown build target: ${release_target}"; \
      exit 1; \
    fi

# copy artifacts to dist mount
CMD bash -c ' \
    mkdir /tmp/grapl && \
    cd /tmp/grapl/ && \
    for service in analyzer-dispatcher \
             generic-subgraph-generator \
             graph-merger \
             metric-forwarder \
             node-identifier \
             node-identifier-retry-handler \
             sysmon-subgraph-generator; do \
      cp /grapl/target/x86_64-unknown-linux-musl/${release_target}/${service} ./bootstrap && \
      zip -9 --quiet --display-globaldots /dist/${service}.zip ./bootstrap; \
    done'

#
# Distribution images
#

FROM alpine AS grapl-rust-dist
RUN apk add --no-cache libgcc

FROM grapl-rust-dist as grapl-analyzer-dispatcher
ARG release_target="debug"
COPY --from=grapl-rust-build /grapl/target/x86_64-unknown-linux-musl/${release_target}/analyzer-dispatcher /
CMD /analyzer-dispatcher

FROM grapl-rust-dist as grapl-generic-subgraph-generator
ARG release_target="debug"
COPY --from=grapl-rust-build /grapl/target/x86_64-unknown-linux-musl/${release_target}/generic-subgraph-generator /
CMD /generic-subgraph-generator

FROM grapl-rust-dist as grapl-graph-merger
ARG release_target="debug"
COPY --from=grapl-rust-build /grapl/target/x86_64-unknown-linux-musl/${release_target}/graph-merger /
CMD /graph-merger

FROM grapl-rust-dist as grapl-metric-forwarder
ARG release_target="debug"
COPY --from=grapl-rust-build /grapl/target/x86_64-unknown-linux-musl/${release_target}/metric-forwarder /
CMD /metric-forwarder

FROM grapl-rust-dist AS grapl-node-identifier
ARG release_target="debug"
COPY --from=grapl-rust-build /grapl/target/x86_64-unknown-linux-musl/${release_target}/node-identifier /
CMD /node-identifier

FROM grapl-rust-dist AS grapl-node-identifier-retry-handler
ARG release_target="debug"
COPY --from=grapl-rust-build /grapl/target/x86_64-unknown-linux-musl/${release_target}/node-identifier-retry-handler /
CMD /node-identifier-retry-handler

FROM grapl-rust-dist AS grapl-sysmon-subgraph-generator
ARG release_target="debug"
COPY --from=grapl-rust-build /grapl/target/x86_64-unknown-linux-musl/${release_target}/sysmon-subgraph-generator /
CMD /sysmon-subgraph-generator
