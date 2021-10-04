#
# Makefile for building Engagement View
#

.DEFAULT_GOAL := build

DOCKER_RUN := docker run \
		--rm \
		--interactive \
		--tty \
		--user "$(shell id -u):$(shell id -g)" \
		--workdir /engagement_view \
		--env HOME=/engagement_view \
		--mount type=volume,source=grapl-yarn-state-engagement-view,target=/engagement_view/.yarn/state \
		--mount type=volume,source=grapl-node-modules-engagement-view,target=/engagement_view/node_modules \
		--mount type=bind,source="$(shell pwd)",target=/engagement_view \
		grapl/engagement-view-build-env

.PHONY: build-env-image
build-env-image:
	docker buildx build \
		--tag grapl/engagement-view-build-env \
		- < build-env.Dockerfile

.PHONY: node_modules
node_modules: build-env-image
	$(DOCKER_RUN) yarn install

.PHONY: build
build: node_modules
	$(DOCKER_RUN) yarn build

.PHONY: test
test: node_modules
	$(DOCKER_RUN) yarn test --coverage --watchAll=false --coverageDirectory=/engagement_view/coverage/

.PHONY: clean
clean:
	docker volume rm \
		grapl-node-modules-engagement-view \
		grapl-yarn-state-engagement-view