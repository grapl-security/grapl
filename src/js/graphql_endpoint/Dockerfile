FROM node:alpine3.10 as build
WORKDIR /lambda

RUN apk upgrade && apk add zip
RUN apk add python
RUN apk add --update alpine-sdk
RUN apk add libffi-dev openssl-dev
RUN apk --no-cache --update add build-base

COPY package.json .
COPY package-lock.json .

RUN npm i

RUN mkdir -p /lambda/modules/
COPY modules/* ./modules/

COPY server.js .
RUN zip --quiet -9r /lambda.zip .

#FROM scratch

#COPY --from=0 /lambda.zip /

