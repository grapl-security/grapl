FROM node:alpine3.10 AS engagement-view-build
COPY . .
RUN yarn install
RUN yarn build
#RUN npm rebuild --build-from-source grpc

FROM syntaqx/serve AS grapl-engagement-view
COPY --from=engagement-view-build ./build/ /var/www

