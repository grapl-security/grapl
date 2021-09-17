#
# Makefile for building Engagement View
#

.DEFAULT_GOAL := build

ASSET_FILTER = -not -path "./node_modules/*" -not -path "./.yarn/*" -not -path "./build/*" -not -path "./build"
ASSET_DIRS = $(shell find . -type d $(ASSET_FILTER))
ASSET_FILES = $(shell find . -type f $(ASSET_FILTER))

DOCKER_RUN := docker run \
		--rm \
		--interactive \
		--tty \
		--user "$(shell id -u):$(shell id -g)" \
		--workdir /engagement_view \
		--mount type=bind,source="$(shell pwd)",target=/engagement_view \
		--env HOME=/engagement_view \
		node:16-buster-slim

node_modules: package.json package-lock.json
	$(DOCKER_RUN) yarn install

build: node_modules $(ASSET_DIRS) $(ASSET_FILES)
	$(DOCKER_RUN) yarn build

.PHONY: test
test: node_modules
	$(DOCKER_RUN) yarn test --coverage --watchAll=false --coverageDirectory=/engagement_view/coverage/
