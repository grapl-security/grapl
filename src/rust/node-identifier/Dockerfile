FROM alpine AS grapl-node-identifier
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/${release_target}/node-identifier /bootstrap
CMD ["./bootstrap"]

FROM alpine AS grapl-node-identifier-retry-handler
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/${release_target}/node-identifier-retry-handler /bootstrap
CMD ["./bootstrap"]
