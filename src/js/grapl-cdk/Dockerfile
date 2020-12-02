# NOTE: this build is half-baked, and primarily supplies us with typechecks at CI time.
# I haven't yet figured out a way to run the tests, primarily owing to the fact we need the ZIP'd artifacts
# of other services/builds.

FROM node:alpine3.10 AS grapl-cdk-build
WORKDIR /home/grapl
# no-op the base image cmd, so it doesn't launch a Node repl
CMD :
RUN apk add bash
RUN npm install --global aws-cdk

# Takes a little while.
# I do this separately from the below 'copy .' to take advantage of docker layer caching
COPY ./node_modules /home/grapl/grapl-cdk/node_modules

COPY . /home/grapl/grapl-cdk

WORKDIR /home/grapl/grapl-cdk
RUN npm install
# does typechecking, which is nice
RUN cd /home/grapl/grapl-cdk && npm run build