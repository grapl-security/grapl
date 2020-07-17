FROM node:alpine3.10 AS engagement-view-build
WORKDIR /grapl
COPY . .
RUN yarn install
RUN CI=true yarn test
RUN yarn build

FROM syntaqx/serve AS grapl-engagement-view
COPY --from=engagement-view-build /grapl/build/ /var/www
