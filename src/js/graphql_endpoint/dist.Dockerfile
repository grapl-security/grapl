FROM gcr.io/distroless/nodejs:debug

WORKDIR /graphql_endpoint

COPY ts_compiled /graphql_endpoint
COPY start_potentially_with_debugger.sh start_potentially_with_debugger.sh

USER nonroot

# The Entrypoint is "node":
# https://github.com/GoogleContainerTools/distroless/blob/main/nodejs/README.md
CMD [ "server.js" ]

