FROM node:16-buster-slim

WORKDIR /graphql_endpoint

COPY ts_compiled /graphql_endpoint
COPY start_potentially_with_debugger.sh start_potentially_with_debugger.sh

USER nobody

CMD [ "node", "server.js" ]