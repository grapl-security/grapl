FROM alpine AS grapl-node-identifier
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/x86_64-unknown-linux-musl/${release_target}/node-identifier /
CMD ["./node-identifier"]

FROM alpine AS grapl-node-identifier-retry-handler
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/x86_64-unknown-linux-musl/${release_target}/node-identifier-retry-handler /
CMD ["./node-identifier-retry-handler"]
