FROM alpine AS grapl-sysmon-subgraph-generator
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/${release_target}/sysmon-subgraph-generator /bootstrap
CMD ["./bootstrap"]
