FROM node:alpine3.10 AS engagement-view-deps

RUN apk add bash yarn
WORKDIR /grapl

# install deps as separate steps to leverage build cache
COPY package.json package.json
COPY yarn.lock yarn.lock
RUN yarn install

# now copy all sources
COPY . .

# create production build
################################################################################
FROM engagement-view-deps AS engagement-view-build

# build sources
RUN yarn build

# no-op the base image cmd, so it doesn't launch a Node repl
CMD :

# create an image that we can deploy from locally
################################################################################
# py chosen because it has `apt` for wait-for-it, and we need pip to install awscli
FROM python:3.7-slim-buster AS engagement-view-local-deploy

SHELL ["/bin/bash", "-c"]

RUN apt-get update && apt-get -y install --no-install-recommends \
        wait-for-it

RUN python3 -m pip install awscli

COPY --from=engagement-view-build /grapl /grapl
WORKDIR /grapl