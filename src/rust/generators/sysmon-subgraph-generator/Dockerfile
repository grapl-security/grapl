FROM alpine AS grapl-sysmon-subgraph-generator
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/x86_64-unknown-linux-musl/${release_target}/sysmon-subgraph-generator /
CMD ["./sysmon-subgraph-generator"]
