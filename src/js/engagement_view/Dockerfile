FROM node:alpine3.10 AS engagement-view-deps

RUN apk add bash yarn
WORKDIR /grapl

# install deps as separate steps to leverage build cache
COPY js/engagement_view/package.json package.json
COPY js/engagement_view/yarn.lock yarn.lock
COPY js/engagement_view/.yarnrc.yml .yarnrc.yml
COPY js/engagement_view/.yarn/releases .yarn/releases

RUN yarn set version berry
RUN --mount=type=cache,target=.yarn/cache/ yarn install

# now copy all sources
COPY js/engagement_view .

# create production build
################################################################################
FROM engagement-view-deps AS engagement-view-build

# build sources
RUN --mount=type=cache,target=.yarn/cache/ yarn add react-scripts
RUN --mount=type=cache,target=.yarn/cache/ yarn build

# no-op the base image cmd, so it doesn't launch a Node repl
CMD :

# create an image that we can deploy from locally
################################################################################
# we need pip to install awscli
FROM python:3.7-slim-buster AS engagement-view-local-deploy

SHELL ["/bin/bash", "-c"]

RUN python3 -m pip install awscli

COPY --from=engagement-view-build /grapl /grapl
WORKDIR /grapl

