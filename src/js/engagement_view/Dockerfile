FROM node:alpine3.10 AS engagement-view-build
RUN apk add bash

ARG USERID="1000:1000"
USER $USERID
WORKDIR /grapl

COPY . .

RUN yarn install
RUN yarn build

FROM syntaqx/serve AS grapl-engagement-view
COPY --from=engagement-view-build /grapl/build/ /var/www
