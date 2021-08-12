FROM node:16-buster-slim AS engagement-view-deps

WORKDIR /grapl

# install deps as separate steps to leverage build cache
COPY package.json package.json
COPY yarn.lock yarn.lock
COPY .yarnrc.yml .yarnrc.yml
COPY .yarn/releases .yarn/releases

RUN yarn set version berry
RUN yarn install

# now copy all sources
COPY . .

# create production build
################################################################################
FROM engagement-view-deps AS engagement-view-build

# build sources
RUN yarn build


# create an image that we can deploy from locally
################################################################################
# we need pip to install awscli
FROM python:3.7-slim-buster AS engagement-view-local-deploy

RUN python3 -m pip install awscli

COPY --from=engagement-view-build /grapl /grapl
WORKDIR /grapl

CMD ["./upload_local.sh"]
