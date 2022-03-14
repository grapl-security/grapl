FROM node:16-bullseye

SHELL ["/bin/bash", "-c"]

# We do the following things to the default node container:
#1. Upgrade the yarn version
#2. We set the permissions for the docker mount points

########## Upgrade Yarn Version ############################
# The official node containers all use yarnv1, which is old and not really supported anymore. Therefore we're setting
# yarn to the latest stable v3 version explicitly. With this we'll also be able to enable features like zero installs

# TODO switch to corepack once
#`typescript@patch:typescript@npm%3A4.6.2#~builtin<compat/typescript>::version=4.6.2&hash=493e53: Cannot apply hunk #11`
# error is fixed. Apparently corepack applies a strict setting, which yarn set doesn't.

ENV YARN_VERSION 3.1.1

# Don't think this is necessarily an issue for us:
# hadolint ignore=SC2174
RUN yarn set version ${YARN_VERSION}

########## Set docker mount points mode ###################
# Manually create Docker volume mount points so we can set the mode
# to make them a+w.
RUN mkdir --parents \
    /engagement_view/.yarn/state \
    /engagement_view/node_modules \
    && chmod --recursive 777 \
    /engagement_view/.yarn/state \
    /engagement_view/node_modules

VOLUME /engagement_view/.yarn/state
VOLUME /engagement_view/node_modules
