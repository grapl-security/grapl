FROM node as build
WORKDIR /lambda

RUN apt-get update && apt-get install -y zip
COPY package.json .
COPY package-lock.json .

RUN npm i

RUN mkdir -p /lambda/modules/
COPY modules/* ./modules/

COPY server.js .
RUN zip --quiet -9r /lambda.zip .

#FROM scratch

#COPY --from=0 /lambda.zip /

