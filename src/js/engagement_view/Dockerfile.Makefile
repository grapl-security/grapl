FROM node:alpine3.10 AS engagement-view-build

RUN apk add bash
WORKDIR /grapl

# install deps as separate steps to leverage build cache
COPY package.json package.json
COPY package-lock.json package-lock.json
RUN yarn install

# copy and build sources
COPY . .

RUN yarn build

# set default command to run tests
CMD CI=true yarn test

#
# deploy
#

FROM syntaqx/serve AS grapl-engagement-view

COPY --from=engagement-view-build /grapl/build/ /var/www

# no-op the base image cmd, so it doesn't launch a Node repl
CMD :
