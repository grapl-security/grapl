# syntax=docker/dockerfile:1.4

# grapl-typescript-base
################################################################################
# We're using node:18-bullseye because it allows sudo acccess.
FROM node:18-bullseye AS grapl-typescript-base
SHELL ["/bin/bash", "-o", "pipefail", "-c"]


# Allow grapl user to su to root. This requires some shennanigans since Debian slim doesn't include sudo by default
#RUN groupadd wheel \
#    && usermod -aG wheel root \
#    && usermod -aG wheel grapl \
#    && echo "auth       sufficient pam_wheel.so trust" | tee /etc/pam.d/su \
#    && echo "permissions" ls -l /usr/bin/su

# Uncomment to get the versions for pins
#RUN dpkg-query -l && exit 5

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

RUN yarn install \
    && yarn build \
    && yarn cache clean \
    && yarn playwright install --with-deps

RUN adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl

USER grapl



CMD ["yarn e2eTest"]