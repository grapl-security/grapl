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
WORKDIR /home/grapl/lambda
COPY --chown=grapl js/graphql_endpoint/tsconfig.json tsconfig.json
COPY --chown=grapl js/graphql_endpoint/package.json package.json
COPY --chown=grapl js/graphql_endpoint/package-lock.json package-lock.json
RUN npm install
RUN rm -rf node_modules/grpc/build/
RUN cp -r ./node_modules/ ./ts_compiled/

# Copy graphql sources
COPY --chown=grapl js/graphql_endpoint/modules modules
COPY --chown=grapl js/graphql_endpoint/server.ts server.ts

# This compiles the typescript and spits out the built js files to `lambda/ts_compiled'
RUN npx tsc

WORKDIR /home/grapl

# zip
FROM graphql-endpoint-build AS graphql-endpoint-zip

RUN cd lambda/ts_compiled && \
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

WORKDIR /home/grapl/lambda/ts_compiled

COPY --chown=grapl js/graphql_endpoint/package.json package.json
COPY --chown=grapl js/graphql_endpoint/package-lock.json package-lock.json

CMD yarn start server