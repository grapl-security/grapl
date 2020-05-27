FROM alpine AS grapl-node-identifier
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/${release_target}/node-identifier /
CMD ["./node-identifier"]

FROM alpine AS grapl-node-identifier-retry-handler
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/${release_target}/node-identifier-retry-handler /
CMD ["./node-identifier-retry-handler"]
