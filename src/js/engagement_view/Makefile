#
# Makefile for building Engagement View
#

.DEFAULT_GOAL := build
# Disable implicit rules, not needed for our use case
.SUFFIXES:

DOCKER_RUN := docker run \
		--rm \
		--user "$(shell id -u):$(shell id -g)" \
		--workdir /engagement_view \
		--env HOME=/engagement_view \
		--mount type=volume,source=grapl-yarn-state-engagement-view,target=/engagement_view/.yarn/state \
		--mount type=volume,source=grapl-node-modules-engagement-view,target=/engagement_view/node_modules \
		--mount type=bind,source="$(shell pwd)",target=/engagement_view \
		grapl/engagement-view-build-env

# These are used to declare Makefile dependencies.
# If an ASSET_DIRS/FILES has changed, rebuild! Otherwise we're likely up to date!
ASSET_FILTER = -not -path "./node_modules/*" -not -path "./.yarn/*" -not -path "./build/*" -not -path "./build" -not -path "./.npm/*"
ASSET_DIRS = $(shell find . -type d $(ASSET_FILTER))
ASSET_FILES = $(shell find . -type f $(ASSET_FILTER))

.PHONY: build-env-image
build-env-image:
	docker buildx build \
		--tag grapl/engagement-view-build-env \
		- < build-env.Dockerfile

# This will run every time, because all PHONYs (e.g. build-env-image) run every time.
node_modules: build-env-image package.json yarn.lock
	$(DOCKER_RUN) yarn install

build: node_modules $(ASSET_DIRS) $(ASSET_FILES)
	$(DOCKER_RUN) yarn build

.PHONY: test
test: node_modules
	$(DOCKER_RUN) yarn test --coverage --watchAll=false --coverageDirectory=/engagement_view/coverage/

.PHONY: clean
clean:
	docker volume rm \
		grapl-node-modules-engagement-view \
		grapl-yarn-state-engagement-view
