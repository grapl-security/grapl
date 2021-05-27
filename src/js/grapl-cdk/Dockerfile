# NOTE: this build is half-baked, and primarily supplies us with typechecks at CI time.
# I haven't yet figured out a way to run the tests, primarily owing to the fact we need the ZIP'd artifacts
# of other services/builds.

FROM node:alpine3.10 AS grapl-cdk-build

# no-op the base image cmd, so it doesn't launch a Node repl
CMD :
RUN apk add bash
RUN npm install --global aws-cdk

# Grab all dependencies
WORKDIR /home/grapl
COPY js/grapl-cdk/package.json src/js/grapl-cdk/package.json
COPY js/grapl-cdk/package-lock.json src/js/grapl-cdk/package-lock.json
WORKDIR /home/grapl/src/js/grapl-cdk
RUN npm install

# Copy in the rest of the source
WORKDIR /home/grapl
COPY js/grapl-cdk src/js/grapl-cdk

# does typechecking, which is nice
WORKDIR /home/grapl/src/js/grapl-cdk
RUN npm run build

# test
######################################################
FROM grapl-cdk-build AS grapl-cdk-test

# We need the Rust Dockerfile to run the tests
WORKDIR /home/grapl
COPY rust/Dockerfile src/rust/Dockerfile
COPY python/Dockerfile src/python/Dockerfile

# Make fake zip files to appease the CDK tests
WORKDIR /home/grapl/src/js/grapl-cdk/zips
RUN touch \
  dgraph-ttl-latest.zip \
  engagement-creator-latest.zip \
  engagement-edge-latest.zip \
  engagement-ux-latest.zip \
  graphql-endpoint-latest.zip \
  metric-forwarder-latest.zip \
  model-plugin-deployer-latest.zip \
  ux-router-latest.zip \
  provisioner-latest.zip \
  e2e-test-runner-latest.zip

WORKDIR /home/grapl/src/js/grapl-cdk
