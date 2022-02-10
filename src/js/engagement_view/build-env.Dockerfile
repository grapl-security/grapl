FROM node:16-bullseye-slim

SHELL ["/bin/bash", "-c"]

# The official node containers all use yarnv1, which is old and not really supported anymore. Therefore we're setting
# yarn to the latest stable v3 version explicitly. With this we'll also be able to enable features like zero installs
ENV YARN_VERSION 3.1.1
RUN yarn set version $YARN_VERSION

# Manually create Docker volume mount points so we can set the mode
# to make them a+w.
#
# Don't think this is necessarily an issue for us:
# hadolint ignore=SC2174
RUN mkdir --mode=777 --parents /engagement_view/{.yarn/state,node_modules}
