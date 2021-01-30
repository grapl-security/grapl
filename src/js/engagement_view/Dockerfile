FROM node:alpine3.10 AS engagement-view-deps

RUN apk add bash
WORKDIR /grapl

# install deps as separate steps to leverage build cache
COPY package.json package.json
COPY yarn.lock yarn.lock
RUN yarn install

# now copy all sources
COPY . .

#
# create production build
#
FROM engagement-view-deps AS engagement-view-build

# build sources
RUN yarn build

# no-op the base image cmd, so it doesn't launch a Node repl
CMD :
