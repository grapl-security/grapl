#
# The Grapl Rust Dockerfile
#
# This file is where we build and create all our Rust images. This
# is to avoid having to build the project dependencies multiple times
# (e.g. once for each individual binary). Instead, we build all the
# Rust code once and then pull out all the binaries into their own
# images. Additionally, we leverage cargo vendor and docker's cache
# to minimize the amount of compilation required during local builds.
#
# See:
#  https://benjamincongdon.me/blog/2019/12/04/Fast-Rust-Docker-Builds-with-cargo-vendor/
#  https://github.com/rust-lang/cargo/issues/2644
#

FROM rust:alpine AS grapl-rust-build
ARG release_target="debug"
RUN apk add --no-cache musl-dev libc6-compat protoc
ENV PROTOC /usr/bin/protoc
ENV PROTOC_INCLUDE /usr/include
RUN adduser -D --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
ENV USER grapl
WORKDIR /home/grapl
# build a fake project structure, but copy in the real Cargo.tomls
RUN cargo new --bin analyzer-dispatcher
COPY --chown=grapl analyzer-dispatcher/Cargo.toml analyzer-dispatcher/Cargo.toml
RUN cargo new --lib derive-dynamic-node
COPY --chown=grapl derive-dynamic-node/Cargo.toml derive-dynamic-node/Cargo.toml
RUN cargo new --bin generic-subgraph-generator
COPY --chown=grapl generic-subgraph-generator/Cargo.toml generic-subgraph-generator/Cargo.toml
#RUN cargo new --lib graph-descriptions
#COPY --chown=grapl graph-descriptions/Cargo.toml graph-descriptions/Cargo.toml
#RUN cargo new --lib graph-generator-lib
#COPY --chown=grapl graph-generator-lib/Cargo.toml graph-generator-lib/Cargo.toml
RUN cargo new --bin graph-merger
COPY --chown=grapl graph-merger/Cargo.toml graph-merger/Cargo.toml
# RUN cargo new --lib grapl-config
# COPY --chown=grapl grapl-config/Cargo.toml grapl-config/Cargo.toml

# this is gross, we have to do it because node-identifier's Cargo.toml has [[bin]] sections
RUN cargo new --bin node-identifier && \
    mkdir -p node-identifier/src/bin && \
    cp node-identifier/src/main.rs node-identifier/src/bin/node-identifier.rs && \
    cp node-identifier/src/main.rs node-identifier/src/bin/node-identifier-retry-handler.rs
COPY --chown=grapl node-identifier/Cargo.toml node-identifier/Cargo.toml
RUN cargo new --bin sysmon-subgraph-generator
COPY --chown=grapl sysmon-subgraph-generator/Cargo.toml sysmon-subgraph-generator/Cargo.toml

# copy in the top-level Cargo.toml and Cargo.lock
COPY --chown=grapl Cargo.lock .
COPY --chown=grapl Cargo.toml .

# grapl-config, graph-descriptions, and graph-generator-lib are special because
# other things depend on them, so we have to build them explicitly now
COPY --chown=grapl ./grapl-config grapl-config
COPY --chown=grapl ./graph-descriptions graph-descriptions
COPY --chown=grapl ./graph-generator-lib graph-generator-lib
RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build -p grapl-config
RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build -p grapl-graph-descriptions
RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build -p graph-generator-lib
RUN if [ "${release_target}" == "release" ]; \
        then RUSTFLAGS="-C target-feature=-crt-static" cargo build --release -p grapl-config && \
        RUSTFLAGS="-C target-feature=-crt-static" cargo build --release -p grapl-graph-descriptions && \
        RUSTFLAGS="-C target-feature=-crt-static" cargo build --release -p graph-generator-lib; \
    fi

# vendor and build the deps
RUN mkdir /home/grapl/.cargo
RUN cargo vendor --versioned-dirs --locked > /home/grapl/.cargo/config
# FIXME: cargo test instead of cargo build once the tests work
RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build
RUN if [ "${release_target}" == "release" ]; \
      then RUSTFLAGS="-C target-feature=-crt-static" cargo build --release; \
    fi

# remove all the fake project crap
RUN rm -r analyzer-dispatcher && \
    rm -r derive-dynamic-node && \
    rm -r generic-subgraph-generator && \
    rm -r graph-merger && \
    rm -r node-identifier && \
    rm -r sysmon-subgraph-generator

# copy in the source tree (we do this part separately s.t. docker caches the deps)
COPY --chown=grapl ./analyzer-dispatcher analyzer-dispatcher
COPY --chown=grapl ./derive-dynamic-node derive-dynamic-node
COPY --chown=grapl ./generic-subgraph-generator generic-subgraph-generator
COPY --chown=grapl ./graph-merger graph-merger
COPY --chown=grapl ./node-identifier node-identifier
COPY --chown=grapl ./sysmon-subgraph-generator sysmon-subgraph-generator

# build everything
# FIXME: cargo test instead of cargo build once the tests work
RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build
RUN if [ "${release_target}" == "release" ]; \
      then RUSTFLAGS="-C target-feature=-crt-static" cargo build --release; \
    fi

RUN mkdir -p ~/bin/${release_target}
RUN cp target/${release_target}/analyzer-dispatcher ~/bin/${release_target}/analyzer-dispatcher && \
    cp target/${release_target}/graph-merger ~/bin/${release_target}/graph-merger && \
    cp target/${release_target}/node-identifier ~/bin/${release_target}/node-identifier && \
    cp target/${release_target}/node-identifier-retry-handler ~/bin/${release_target}/node-identifier-retry-handler && \
    cp target/${release_target}/sysmon-subgraph-generator ~/bin/${release_target}/sysmon-subgraph-generator

# analyzer-dispatcher
FROM alpine AS grapl-analyzer-dispatcher
ARG release_target="release"
COPY --from=grapl-rust-build /home/grapl/bin/${release_target}/analyzer-dispatcher /
CMD ["analyzer-dispatcher"]

# graph-merger
FROM alpine AS grapl-graph-merger
ARG release_target="release"
COPY --from=grapl-rust-build /home/grapl/bin/${release_target}/graph-merger /
CMD ["graph-merger"]

# node-identifier
FROM alpine AS grapl-node-identifier
ARG release_target="release"
COPY --from=grapl-rust-build /home/grapl/bin/${release_target}/node-identifier /
CMD ["node-identifier"]

FROM alpine AS grapl-node-identifier-retry-handler
ARG release_target="release"
COPY --from=grapl-rust-build /home/grapl/bin/${release_target}/node-identifier-retry-handler /
CMD ["node-identifier-retry-handler"]

# sysmon-subgraph-generator
FROM alpine AS grapl-sysmon-subgraph-generator
ARG release_target="release"
COPY --from=grapl-rust-build /home/grapl/bin/${release_target}/sysmon-subgraph-generator /
CMD ["sysmon-subgraph-generator"]
