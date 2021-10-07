#
# Makefile for building GraphQL Endpoint
#

.DEFAULT_GOAL := build

DOCKER_RUN := docker run \
		--rm \
		--interactive \
		--tty \
		--user "$(shell id -u):$(shell id -g)" \
		--workdir /graphql_endpoint \
		--env HOME=/graphql_endpoint \
		--mount type=volume,source=grapl-yarn-state-graphql-endpoint,target=/graphql_endpoint/.yarn/state \
		--mount type=volume,source=grapl-node-modules-graphql-endpoint,target=/graphql_endpoint/node_modules \
		--mount type=bind,source="$(shell pwd)",target=/graphql_endpoint \
		grapl/graphql-endpoint-build-env

.PHONY: build-env-image
build-env-image:
	docker buildx build \
		--tag grapl/graphql-endpoint-build-env \
		- < build-env.Dockerfile

.PHONY: node_modules
node_modules: build-env-image
	$(DOCKER_RUN) yarn install

.PHONY: build
build: node_modules
	$(DOCKER_RUN) npx tsc && cp -r ./node_modules ./ts_compiled/node_modules
	docker buildx bake

.PHONY: test
test: node_modules
	$(DOCKER_RUN) yarn test --coverage --watchAll=false --coverageDirectory=/graphql_endpoint/coverage/

.PHONY: clean
clean:
	docker volume rm \
		grapl-node-modules-graphql-endpoint \
		grapl-yarn-state-graphql-endpoint

