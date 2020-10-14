FROM node:alpine3.10 AS engagement-view-build
RUN apk add bash
WORKDIR /grapl
COPY . .
RUN yarn install
RUN yarn build

FROM syntaqx/serve AS grapl-engagement-view
COPY --from=engagement-view-build /grapl/build/ /var/www

# no-op the base image cmd, so it doesn't launch a Node repl
CMD :
