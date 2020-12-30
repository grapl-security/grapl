FROM node:12.18-buster-slim AS graphql-endpoint-build

RUN apt-get update && apt-get -y install --no-install-recommends \
    build-essential \
    libffi-dev \
    libssl-dev \
    python3 \
    zip

RUN mkdir -p lambda

# Install the dependencies separately to leverage Docker cache
COPY package.json lambda/package.json
COPY package-lock.json lambda/package-lock.json
RUN cd lambda && npm i
RUN rm -rf lambda/node_modules/grpc/build/

# Copy graphql sources
COPY modules lambda/modules
COPY server.js lambda/server.js

CMD ORIG_DIR=$(pwd); \
    cd /lambda && \
    zip -qr "${ORIG_DIR}/lambda.zip" ./

# deploy
FROM node:12.18-buster-slim AS graphql-endpoint-deploy

RUN adduser --disabled-password --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
WORKDIR /home/grapl

COPY --chown=grapl --from=graphql-endpoint-build lambda /home/grapl
COPY --chown=grapl package.json lambda/package.json
COPY --chown=grapl package-lock.json lambda/package-lock.json

CMD yarn start server