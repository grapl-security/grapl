# NOTE: this build is half-baked, and primarily supplies us with typechecks at CI time.
# I haven't yet figured out a way to run the tests, primarily owing to the fact we need the ZIP'd artifacts
# of other services/builds.

FROM node:alpine3.10 AS grapl-cdk-build
WORKDIR /home/grapl
COPY . .
RUN npm install
RUN npm install --global aws-cdk

# does typechecking, which is nice
RUN npm run build

# more useful than the Node shell
RUN apk add bash
CMD "/bin/bash"