FROM alpine AS grapl-analyzer-dispatcher
RUN apk add --no-cache libgcc
ARG release_target="debug"
COPY --from=grapl/grapl-rust-src-build /home/grapl/target/${release_target}/analyzer-dispatcher /
CMD ["./analyzer-dispatcher"]
