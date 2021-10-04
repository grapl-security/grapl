# images for running services
################################################################################
# More information about the base image used here can be found at: 
# https://github.com/GoogleContainerTools/distroless/blob/main/cc/README.md.
# For debugging see: https://github.com/GoogleContainerTools/distroless#debug-images

# NOTE: we're using the debug containers at the moment so we have a
# shell; this lets us inject our Pulumi outputs in Local Grapl. If
# not for that, we could use the standard non-debug images.
FROM gcr.io/distroless/cc:debug AS rust-dist

USER nonroot

# analyzer-dispatcher
FROM rust-dist AS analyzer-dispatcher

COPY /analyzer-dispatcher /
ENTRYPOINT ["/analyzer-dispatcher"]

# generic-subgraph-generator
FROM rust-dist AS generic-subgraph-generator

COPY /generic-subgraph-generator /
ENTRYPOINT ["/generic-subgraph-generator"]

# graph-merger
FROM rust-dist AS graph-merger

COPY /graph-merger /
ENTRYPOINT ["/graph-merger"]

# node-identifier
FROM rust-dist AS node-identifier

COPY /node-identifier /
ENTRYPOINT ["/node-identifier"]

# node-identifier-retry
FROM rust-dist AS node-identifier-retry

COPY /node-identifier-retry /
ENTRYPOINT ["/node-identifier-retry"]

# sysmon-generator
FROM rust-dist AS sysmon-generator

COPY /sysmon-generator /
ENTRYPOINT ["/sysmon-generator"]

# osquery-generator
FROM rust-dist AS osquery-generator

COPY /osquery-generator /
ENTRYPOINT ["/osquery-generator"]

# grapl-web-ui
FROM rust-dist AS grapl-web-ui

COPY /grapl-web-ui /
COPY frontend /frontend
ENTRYPOINT ["/grapl-web-ui"]
