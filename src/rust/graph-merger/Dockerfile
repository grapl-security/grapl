FROM alpine AS grapl-graph-merger
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/${release_target}/graph-merger /
CMD ["./graph-merger"]
