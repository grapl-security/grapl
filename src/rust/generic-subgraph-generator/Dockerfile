FROM alpine AS grapl-generic-subgraph-generator
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/${release_target}/generic-subgraph-generator /
CMD ["./generic-subgraph-generator"]
