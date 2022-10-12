# syntax=docker/dockerfile:1.4

# grapl-typescript-base
################################################################################
FROM node:18-bullseye-slim AS grapl-typescript-base
SHELL ["/bin/bash", "-c"]

RUN adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl
# Uncomment to get the versions for pins
#RUN dpkg-query -l && exit 5

USER grapl
WORKDIR /home/grapl
ENTRYPOINT ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]

# integration-tests
################################################################################
FROM grapl-typescript-base AS integration-tests
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# TODO: make sure yarn version matches package.json
ENV YARN_VERSION 3.2.3
RUN yarn policies set-version $YARN_VERSION

COPY --chown=grapl . frontend
WORKDIR "/grapl/src/js/frontend"

RUN yarn install \
    && yarn build \
    && yarn cache clean

CMD ["yarn test --watchAll=false"]