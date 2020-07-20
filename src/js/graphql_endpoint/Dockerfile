FROM node:12.18-buster-slim AS grapl-graphql-endpoint-deps
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends build-essential libffi-dev libssl-dev python3
RUN adduser --disabled-password --gecos '' --home /home/grapl --shell /bin/bash grapl
WORKDIR /home/grapl
RUN mkdir -p lambda
COPY --chown=grapl package.json lambda/package.json
COPY --chown=grapl package-lock.json lambda/package-lock.json
RUN cd lambda && npm i

FROM node:12.18-buster-slim AS grapl-graphql-endpoint-build
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends zip
RUN adduser --disabled-password --gecos '' --home /home/grapl --shell /bin/bash grapl
WORKDIR /home/grapl
RUN mkdir -p lambda
COPY --chown=grapl --from=grapl-graphql-endpoint-deps /home/grapl/lambda/node_modules lambda/node_modules/
RUN rm -rf lambda/node_modules/grpc/build/
RUN mkdir -p lambda/modules/
COPY modules lambda/modules
# TODO: run tests
COPY server.js lambda/server.js
RUN cd lambda && zip --quiet -9r /home/grapl/lambda.zip .
RUN mkdir -p dist/graphql-endpoint && cp lambda.zip dist/graphql-endpoint/lambda.zip

FROM node:12.18-buster-slim AS grapl-graphql-endpoint
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends unzip
COPY --from=grapl-graphql-endpoint-build /home/grapl/lambda.zip /
RUN unzip -q lambda.zip
COPY --from=grapl-graphql-endpoint-deps /home/grapl/lambda/package.json /package.json
COPY --from=grapl-graphql-endpoint-deps /home/grapl/lambda/package-lock.json /package-lock.json
