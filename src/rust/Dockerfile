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
#  https://stackoverflow.com/questions/54952867/cache-cargo-dependencies-in-a-docker-volume
#  https://github.com/rust-lang/cargo/issues/2644#issuecomment-436907777
#  https://github.com/rust-lang/cargo/issues/2644#issuecomment-460076276
#
# TODO: Investigate using BuildKit to simplify this Dockerfile.

#
# Base image
#

FROM rust:alpine AS grapl-rust-base
RUN apk add --no-cache musl-dev libc6-compat protoc
ENV PROTOC /usr/bin/protoc
ENV PROTOC_INCLUDE /usr/include
RUN adduser -D --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
ENV USER grapl
WORKDIR /home/grapl

#
# Dependency build image
#

FROM grapl-rust-base AS grapl-rust-deps-build
ARG release_target="debug"
USER grapl
ENV USER grapl
WORKDIR /home/grapl
# copy in Cargo.toml for analyzer-dispatcher
RUN mkdir -p analyzer-dispatcher/src
RUN echo 'fn main() {}' > analyzer-dispatcher/src/main.rs
COPY --chown=grapl ./analyzer-dispatcher/Cargo.toml analyzer-dispatcher/
# copy in Cargo.toml for derive-dynamic-node
RUN mkdir -p derive-dynamic-node/src
RUN touch derive-dynamic-node/src/lib.rs
COPY --chown=grapl ./derive-dynamic-node/Cargo.toml derive-dynamic-node/
# copy in Cargo.toml for generic-subgraph-generator
RUN mkdir -p generic-subgraph-generator/src
RUN echo 'fn main() {}' > generic-subgraph-generator/src/main.rs
COPY --chown=grapl ./generic-subgraph-generator/Cargo.toml generic-subgraph-generator/
# copy in Cargo.toml for graph-descriptions
RUN mkdir -p graph-descriptions/src
RUN touch graph-descriptions/src/lib.rs
COPY --chown=grapl ./graph-descriptions/Cargo.toml graph-descriptions/
# copy in Cargo.toml for graph-generator-lib
RUN mkdir -p graph-generator-lib/src
RUN touch graph-generator-lib/src/lib.rs
COPY --chown=grapl ./graph-generator-lib/Cargo.toml graph-generator-lib/
# copy in Cargo.toml for grapl-config
RUN mkdir -p grapl-config/src
RUN touch grapl-config/src/lib.rs
COPY --chown=grapl ./grapl-config/Cargo.toml grapl-config/
# copy in Cargo.toml for graph-merger
RUN mkdir -p graph-merger/src
RUN echo 'fn main() {}' > graph-merger/src/main.rs
COPY --chown=grapl ./graph-merger/Cargo.toml graph-merger/
# copy in Cargo.toml for node-identifier
RUN mkdir -p node-identifier/src/bin
RUN echo 'fn main() {}' > node-identifier/src/bin/node-identifier.rs
RUN echo 'fn main() {}' > node-identifier/src/bin/node-identifier-retry-handler.rs
COPY --chown=grapl ./node-identifier/Cargo.toml node-identifier/
# copy in Cargo.toml for sysmon-subgraph-generator
RUN mkdir -p sysmon-subgraph-generator/src
RUN echo 'fn main() {}' > sysmon-subgraph-generator/src/main.rs
COPY --chown=grapl ./sysmon-subgraph-generator/Cargo.toml sysmon-subgraph-generator/
# copy in the top-level Cargo.toml and Cargo.lock
COPY --chown=grapl Cargo.lock .
COPY --chown=grapl Cargo.toml .
# vendor all the deps
RUN mkdir -p /home/grapl/.cargo
RUN cargo vendor --versioned-dirs --locked > /home/grapl/.cargo/config
# build all the deps
RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build
RUN if [ "${release_target}" == "release" ]; \
        then RUSTFLAGS="-C target-feature=-crt-static" cargo build --release; \
    fi

#
# Source build image
#

FROM grapl-rust-base AS grapl-rust-src-build
ARG release_target="debug"
USER grapl
ENV USER grapl
WORKDIR /home/grapl
# copy in the .cargo directory from grapl-rust-deps-build
COPY --from=grapl-rust-deps-build --chown=grapl /home/grapl/.cargo .cargo
# copy in the vendor directory from grapl-rust-deps-build
COPY --from=grapl-rust-deps-build --chown=grapl /home/grapl/vendor vendor
# copy in the target directory from grapl-rust-deps-build
COPY --from=grapl-rust-deps-build --chown=grapl /home/grapl/target target
# "invalidate" all the grapl services, we want to rebuild them here
RUN rm -r target/${release_target}/.fingerprint/analyzer-dispatcher*
RUN rm -r target/${release_target}/.fingerprint/derive-dynamic-node*
RUN rm -r target/${release_target}/.fingerprint/generic-subgraph-generator*
RUN rm -r target/${release_target}/.fingerprint/grapl-graph-descriptions*
RUN rm -r target/${release_target}/.fingerprint/graph-generator-lib*
RUN rm -r target/${release_target}/.fingerprint/grapl-config*
RUN rm -r target/${release_target}/.fingerprint/graph-merger*
RUN rm -r target/${release_target}/.fingerprint/node-identifier*
RUN rm -r target/${release_target}/.fingerprint/sysmon-subgraph-generator*
# copy in the sources
COPY --chown=grapl ./analyzer-dispatcher analyzer-dispatcher
COPY --chown=grapl ./derive-dynamic-node derive-dynamic-node
COPY --chown=grapl ./generic-subgraph-generator generic-subgraph-generator
COPY --chown=grapl ./graph-descriptions graph-descriptions
COPY --chown=grapl ./graph-generator-lib graph-generator-lib
COPY --chown=grapl ./grapl-config grapl-config
COPY --chown=grapl ./graph-merger graph-merger
COPY --chown=grapl ./node-identifier node-identifier
COPY --chown=grapl ./sysmon-subgraph-generator sysmon-subgraph-generator
# copy in the top-level Cargo.toml and Cargo.lock
COPY --chown=grapl Cargo.lock .
COPY --chown=grapl Cargo.toml .
# build everything
# FIXME: cargo test instead of cargo build once the tests work
RUN RUSTFLAGS="-C target-feature=-crt-static" cargo build
RUN if [ "${release_target}" == "release" ]; \
      then RUSTFLAGS="-C target-feature=-crt-static" cargo build --release; \
    fi

#
# Deploy images
#

# base deploy image
FROM alpine AS grapl-deploy-base
ARG release_target="debug"
RUN if [ "${release_target}" == "debug" ]; \
        then apk add --no-cache libgcc; \
    fi

# analyzer-dispatcher deploy image
FROM grapl-deploy-base AS grapl-analyzer-dispatcher
ARG release_target="debug"
COPY --from=grapl-rust-src-build /home/grapl/target/${release_target}/analyzer-dispatcher /
CMD ["analyzer-dispatcher"]

# graph-merger deploy image
FROM grapl-deploy-base AS grapl-graph-merger
ARG release_target="debug"
COPY --from=grapl-rust-src-build /home/grapl/target/${release_target}/graph-merger /
CMD ["graph-merger"]

# node-identifier deploy images
FROM grapl-deploy-base AS grapl-node-identifier
ARG release_target="debug"
COPY --from=grapl-rust-src-build /home/grapl/target/${release_target}/node-identifier /
CMD ["node-identifier"]

FROM grapl-deploy-base AS grapl-node-identifier-retry-handler
ARG release_target="debug"
COPY --from=grapl-rust-src-build /home/grapl/target/${release_target}/node-identifier-retry-handler /
CMD ["node-identifier-retry-handler"]

# sysmon-subgraph-generator deploy image
FROM grapl-deploy-base AS grapl-sysmon-subgraph-generator
ARG release_target="debug"
COPY --from=grapl-rust-src-build /home/grapl/target/${release_target}/sysmon-subgraph-generator /
CMD ["sysmon-subgraph-generator"]
