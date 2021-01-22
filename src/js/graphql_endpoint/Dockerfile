FROM node:12.18-buster-slim AS graphql-endpoint-build

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
COPY --chown=grapl package.json lambda/package.json
COPY --chown=grapl package-lock.json lambda/package-lock.json
RUN cd lambda && npm i
RUN rm -rf lambda/node_modules/grpc/build/

# Copy graphql sources
COPY --chown=grapl modules lambda/modules
COPY --chown=grapl server.js lambda/server.js

# zip
FROM graphql-endpoint-build AS graphql-endpoint-zip

RUN cd lambda && \
    zip -q9r /home/grapl/lambda.zip ./

# deploy
FROM node:12.18-buster-slim AS graphql-endpoint-deploy

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

COPY --chown=grapl package.json package.json
COPY --chown=grapl package-lock.json package-lock.json

CMD yarn start server