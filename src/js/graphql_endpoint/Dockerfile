FROM node:12.18-buster-slim AS grapl-graphql-endpoint-deps
WORKDIR /lambda

RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends build-essential libffi-dev libssl-dev python3

COPY package.json .
COPY package-lock.json .

RUN npm i

FROM node:12.18-buster-slim AS grapl-graphql-endpoint-build
WORKDIR /lambda
COPY --from=grapl-graphql-endpoint-deps /lambda/node_modules node_modules/
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends zip

RUN rm -rf node_modules/grpc/build/
RUN mkdir -p /lambda/modules/
COPY modules/* ./modules/

COPY server.js .
RUN zip --quiet -9r /lambda.zip .

FROM node:12.18-buster-slim AS grapl-graphql-endpoint
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends unzip
COPY --from=grapl-graphql-endpoint-build /lambda.zip /
RUN unzip -q lambda.zip
COPY --from=grapl-graphql-endpoint-deps /lambda/package.json /package.json
COPY --from=grapl-graphql-endpoint-deps /lambda/package-lock.json /package-lock.json
