# syntax=docker/dockerfile:1.4

# grapl-typescript-base
################################################################################
# We're using node:18-bullseye because it allows sudo acccess.
FROM node:18-bullseye AS grapl-typescript-base
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN adduser \
    --disabled-password \
    --gecos '' \
    --home /home/grapl \
    --shell /bin/bash \
    grapl

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
WORKDIR "/home/grapl/frontend"

# Playwright requires root permissions to install browsers.
# In order to prevent the need to grant the grapl user sudo, we install all dependencies with root and then move any
# user-specific directories like the browsers created in ~/.cache/ from root to the grapl user.
# The alternate approach would be to grant the grapl user sudo or su perms.
RUN yarn install \
    && yarn build \
    && yarn cache clean \
    && yarn playwright install --with-deps  \
    && mkdir -p /home/grapl/.cache/ms-playwright \
    && mv "/root/.cache/ms-playwright" "/home/grapl/.cache/" \
    && chown --recursive grapl:grapl  "/home/grapl/"

USER grapl

CMD ["yarn e2eTest"]