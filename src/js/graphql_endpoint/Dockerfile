# Don't forget to update the other `FROM node:` in this file.
# Use the latest Node version here: https://docs.aws.amazon.com/lambda/latest/dg/lambda-nodejs.html
FROM node:14.16-buster-slim AS graphql-endpoint-build  

RUN apt-get update && apt-get -y install --no-install-recommends \
        build-essential \
        libffi-dev \
        libssl-dev \
        python3 \
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
COPY --chown=grapl tsconfig.json tsconfig.json
COPY --chown=grapl package.json package.json
COPY --chown=grapl jest.config.js jest.config.js
COPY --chown=grapl package-lock.json package-lock.json
RUN npm install
RUN rm -rf node_modules/grpc/build/

# Copy graphql sources
COPY --chown=grapl modules modules
COPY --chown=grapl tests tests
COPY --chown=grapl server.ts server.ts

# This compiles the typescript and spits out the built js files to `lambda/ts_compiled'
RUN npx tsc
RUN cp -r ./node_modules ./ts_compiled/node_modules

WORKDIR /home/grapl


# zip
################################################################################
FROM graphql-endpoint-build AS graphql-endpoint-zip

RUN test -d lambda/ts_compiled/node_modules && echo "Okay, found node_modules"
RUN cd lambda/ts_compiled && \
    zip -q9r /home/grapl/graphql-endpoint-lambda.zip ./

# This defines a default command for extracting the metric-forwarder zip to a
# container mount point at /dist
CMD [ "cp", "/home/grapl/graphql-endpoint-lambda.zip", "/dist/" ]


# deploy... this is only used in Local Grapl at the moment
################################################################################
FROM node:14.16-buster-slim AS graphql-endpoint-deploy

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

COPY --chown=grapl package.json package.json
COPY --chown=grapl package-lock.json package-lock.json
COPY --chown=grapl start_potentially_with_debugger.sh start_potentially_with_debugger.sh

CMD yarn start
