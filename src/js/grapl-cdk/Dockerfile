# NOTE: this build is half-baked, and primarily supplies us with typechecks at CI time.
# I haven't yet figured out a way to run the tests, primarily owing to the fact we need the ZIP'd artifacts
# of other services/builds.

FROM node:alpine3.10 AS grapl-cdk-build
WORKDIR /home/grapl/grapl-cdk
# no-op the base image cmd, so it doesn't launch a Node repl
CMD :
RUN apk add bash
RUN npm install --global aws-cdk

# Grab all dependencies
COPY ./package.json ./package.json
COPY ./package-lock.json ./package-lock.json
RUN npm install

# Copy in the rest of the source
COPY . .
# does typechecking, which is nice
RUN npm run build

FROM grapl-cdk-build AS grapl-cdk-test
# Make fake zip files to appease the CDK tests
WORKDIR /home/grapl/grapl-cdk/zips
RUN touch \
  analyzer-dispatcher-latest.zip \
  analyzer-executor-latest.zip \
  dgraph-ttl-latest.zip \
  engagement-creator-latest.zip \
  engagement-edge-latest.zip \
  engagement-ux-latest.zip \
  graph-merger-latest.zip \
  graphql-endpoint-latest.zip \
  metric-forwarder-latest.zip \ 
  model-plugin-deployer-latest.zip \
  node-identifier-latest.zip \
  node-identifier-retry-handler-latest.zip \
  osquery-subgraph-generator-latest.zip \
  swarm-lifecycle-event-handler-latest.zip \
  sysmon-subgraph-generator-latest.zip

WORKDIR /home/grapl/grapl-cdk
