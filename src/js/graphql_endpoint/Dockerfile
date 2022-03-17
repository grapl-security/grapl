# Don't forget to update the other `FROM node:` in this file.
# Use the latest Node version here: https://docs.aws.amazon.com/lambda/latest/dg/lambda-nodejs.html
FROM node:17.3-bullseye-slim AS graphql-endpoint-build

RUN apt-get update \
    && apt-get --yes install --no-install-recommends \
        build-essential=12.9 \
        libffi-dev=3.3-6 \
        libssl-dev=1.1.1k-1+deb11u2 \
        python3=3.9.2-3 \
        zip=3.0-12 \
    && rm -rf /var/lib/apt/lists/* \
    && adduser \
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
RUN npm install \
    && rm -rf node_modules/grpc/build/

# Copy graphql sources
COPY --chown=grapl modules modules
COPY --chown=grapl tests tests
COPY --chown=grapl server.ts server.ts
COPY --chown=grapl tracing.ts tracing.ts

# This compiles the typescript and spits out the built js files to `lambda/ts_compiled'
RUN npx tsc \
    && cp -r ./node_modules ./ts_compiled/node_modules

WORKDIR /home/grapl


# Deploy image
################################################################################
FROM node:17.3-bullseye-slim AS graphql-endpoint-deploy

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

CMD ["yarn", "start"]
