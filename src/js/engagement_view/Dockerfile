ARG TAG=latest

# deps
################################################################################
FROM grapl/grapl-js-base:$TAG AS engagement-view-deps
# install deps as separate steps to leverage build cache
COPY --chown=grapl js/engagement_view/package.json package.json
COPY --chown=grapl js/engagement_view/yarn.lock yarn.lock
COPY --chown=grapl js/engagement_view/.yarnrc.yml .yarnrc.yml
COPY --chown=grapl js/engagement_view/.yarn/releases .yarn/releases
RUN source $NVM_DIR/nvm.sh && yarn set version berry
RUN source $NVM_DIR/nvm.sh && yarn install

# build
################################################################################
FROM engagement-view-deps AS engagement-view-build
COPY --chown=grapl js/engagement_view .
RUN source $NVM_DIR/nvm.sh && yarn build

# deploy
################################################################################
FROM grapl/grapl-js-base:$TAG AS engagement-view
COPY --from=engagement-view-build /home/grapl /home/grapl
