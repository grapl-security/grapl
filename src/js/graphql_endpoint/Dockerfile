FROM node:alpine3.10 AS grapl-graphql-endpoint-build
WORKDIR /lambda

RUN apk upgrade && apk add zip
RUN apk add python
RUN apk add --update alpine-sdk
RUN apk add libffi-dev openssl-dev
RUN apk --no-cache --update add build-base

COPY package.json .
COPY package-lock.json .

RUN npm i
RUN rm -rf node_modules/grpc/build/
RUN mkdir -p /lambda/modules/
COPY modules/* ./modules/

COPY server.js .
RUN zip --quiet -9r /lambda.zip .

FROM node:alpine3.10 AS grapl-graphql-endpoint
RUN apk add --no-cache zip
COPY --from=grapl-graphql-endpoint-build /lambda.zip /
RUN unzip -q lambda.zip
