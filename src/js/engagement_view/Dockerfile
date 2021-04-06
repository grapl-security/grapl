# deps
################################################################################
FROM grapl-js-build AS engagement-view-deps
# install deps as separate steps to leverage build cache
COPY js/engagement_view/package.json package.json
COPY js/engagement_view/yarn.lock yarn.lock
COPY js/engagement_view/.yarnrc.yml .yarnrc.yml
COPY js/engagement_view/.yarn/releases .yarn/releases
RUN yarn set version berry
RUN yarn install

# build
################################################################################
FROM engagement-view-deps AS engagement-view-build
COPY js/engagement_view .
RUN yarn build

# deploy
################################################################################
FROM grapl-js-base AS engagement-view
RUN source /home/grapl/venv/bin/activate && python3 -m pip install awscli
COPY --from=engagement-view-build /home/grapl /home/grapl
