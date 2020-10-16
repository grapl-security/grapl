FROM node:12.18-buster-slim AS grapl-graphql-endpoint-deps
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends build-essential libffi-dev libssl-dev python3

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

RUN mkdir -p lambda
COPY --chown=grapl package.json lambda/package.json
COPY --chown=grapl package-lock.json lambda/package-lock.json
RUN cd lambda && npm i

FROM node:12.18-buster-slim AS grapl-graphql-endpoint-build
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends zip

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

RUN mkdir -p lambda
COPY --chown=grapl --from=grapl-graphql-endpoint-deps /home/grapl/lambda/node_modules lambda/node_modules/
RUN rm -rf lambda/node_modules/grpc/build/
RUN mkdir -p lambda/modules/
COPY modules lambda/modules
COPY server.js lambda/server.js

CMD bash -c ' \
    cd lambda && \
    zip --quiet -9r /home/grapl/lambda.zip . && \
    cp /home/grapl/lambda.zip /dist/graphql-endpoint.zip'

FROM node:12.18-buster-slim AS grapl-graphql-endpoint
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends unzip

COPY --from=grapl-graphql-endpoint-build /home/grapl/lambda/ /

CMD yarn start server