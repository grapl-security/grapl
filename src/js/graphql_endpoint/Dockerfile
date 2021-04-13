# Don't forget to update the other `FROM node:` in this file.
# Use the latest Node version here: https://docs.aws.amazon.com/lambda/latest/dg/lambda-nodejs.html
FROM node:14.16-buster-slim AS graphql-endpoint-build  

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
COPY --chown=grapl js/graphql_endpoint/jest.config.js jest.config.js
COPY --chown=grapl js/graphql_endpoint/package-lock.json package-lock.json
RUN npm install
RUN rm -rf node_modules/grpc/build/

# Copy graphql sources
COPY --chown=grapl js/graphql_endpoint/modules modules
COPY --chown=grapl js/graphql_endpoint/tests tests
COPY --chown=grapl js/graphql_endpoint/server.ts server.ts

# This compiles the typescript and spits out the built js files to `lambda/ts_compiled'
RUN npx tsc
RUN cp -r ./node_modules ./ts_compiled/node_modules

WORKDIR /home/grapl

# zip
FROM graphql-endpoint-build AS graphql-endpoint-zip

RUN test -d lambda/ts_compiled/node_modules && echo "Okay, found node_modules"
RUN cd lambda/ts_compiled && \
    zip -q9r /home/grapl/lambda.zip ./

# deploy... this is only used in Local Grapl at the moment
FROM node:14.16-buster-slim AS graphql-endpoint-deploy

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
COPY --chown=grapl js/graphql_endpoint/start_potentially_with_debugger.sh \
  start_potentially_with_debugger.sh

CMD yarn start