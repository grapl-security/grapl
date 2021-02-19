FROM node:12.18-buster-slim AS graphql-endpoint-build

RUN apt-get update && apt-get -y install --no-install-recommends \
        build-essential \
        libffi-dev \
        libssl-dev \
        python3 \
        wait-for-it \
        zip

RUN adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl

USER grapl
WORKDIR /home/grapl
RUN mkdir -p lambda

# Install the dependencies separately to leverage Docker cache
COPY --chown=grapl js/graphql_endpoint/package.json lambda/package.json
COPY --chown=grapl js/graphql_endpoint/package-lock.json lambda/package-lock.json
RUN cd lambda && npm install
RUN rm -rf lambda/node_modules/grpc/build/

# Copy graphql sources
COPY --chown=grapl js/graphql_endpoint/modules lambda/modules
COPY --chown=grapl js/graphql_endpoint/server.js lambda/server.js

# zip
FROM graphql-endpoint-build AS graphql-endpoint-zip

RUN cd lambda && \
    zip -q9r /home/grapl/lambda.zip ./

# deploy... this is only used in Local Grapl at the moment
FROM node:12.18-buster-slim AS graphql-endpoint-deploy

# Only really needed for Local Grapl
RUN apt-get update && apt-get -y install --no-install-recommends \
    wait-for-it

RUN adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl

USER grapl
WORKDIR /home/grapl

COPY --chown=grapl --from=graphql-endpoint-build /home/grapl/lambda lambda

WORKDIR /home/grapl/lambda

COPY --chown=grapl js/graphql_endpoint/package.json package.json
COPY --chown=grapl js/graphql_endpoint/package-lock.json package-lock.json

CMD yarn start server