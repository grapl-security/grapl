FROM node:alpine3.10 AS grapl-graphql-endpoint-deps
WORKDIR /lambda

RUN apk add python
RUN apk add --update alpine-sdk
RUN apk add libffi-dev openssl-dev
RUN apk --no-cache --update add build-base

COPY package.json .
COPY package-lock.json .

RUN npm i

FROM node:alpine3.10 AS grapl-graphql-endpoint-build
WORKDIR /lambda
COPY --from=grapl-graphql-endpoint-deps /lambda/node_modules node_modules/
RUN apk upgrade && apk add zip

RUN rm -rf node_modules/grpc/build/
RUN mkdir -p /lambda/modules/
COPY modules/* ./modules/

COPY server.js .
RUN zip --quiet -9r /lambda.zip .

FROM node:alpine3.10 AS grapl-graphql-endpoint
RUN apk add --no-cache zip
COPY --from=grapl-graphql-endpoint-build /lambda.zip /
RUN unzip -q lambda.zip
