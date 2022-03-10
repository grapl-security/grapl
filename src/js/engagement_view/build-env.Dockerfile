FROM node:16-bullseye-slim

SHELL ["/bin/bash", "-c"]

# We do the following things to the default node container:
#1. Enable corepack so we use the yarn version set in package.json.
#2. We set the permissions for the docker mount points

########## Set docker mount points mode ###################
# Manually create Docker volume mount points so we can set the mode
# to make them a+w.

# Don't think this is necessarily an issue for us:
# hadolint ignore=SC2174
RUN corepack enable && \
mkdir --mode=777 --parents /engagement_view/{.yarn/state,node_modules}
