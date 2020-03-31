FROM grapl/grapl_rust_base as builder

# this should be passed in via Makefile using build-args
ARG SERVICE_NAME=sysmon-subgraph-generator

WORKDIR /tmp
RUN mkdir ./artifacts
RUN mkdir -p ./home/rust/.rustup/tmp/

COPY ./ ./${SERVICE_NAME}

# if this chown is not present, will receive the below error.
# this chown needs to be run AFTER the files are copied over
# ```
#     Updating crates.io index
# error: failed to open: /tmp/target/release/.cargo-lock
#
# Caused by:
#   Permission denied (os error 13)
# ```
RUN sudo chown -R rust:rust .
RUN sudo chown -R rust:rust /home/rust/.rustup/


RUN cargo build --release --manifest-path ./${SERVICE_NAME}/Cargo.toml