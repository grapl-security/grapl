#
# Makefile for developing using Docker
#

-include .env
TAG ?= latest
PROFILE ?= debug
UID = $(shell id -u)
GID = $(shell id -g)
DOCKER_BUILDX_BAKE_OPTS ?=
ifneq ($(GRAPL_RUST_ENV),)
DOCKER_BUILDX_BAKE_OPTS += --set *.secrets=id=rust_env,src="$(GRAPL_RUST_ENV)"
endif
export

export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

DOCKER_BUILDX_BAKE := docker buildx bake $(DOCKER_BUILDX_BAKE_OPTS)

PYTHON_UNIT_TESTS := \
	grapl-graph-descriptions-test \
	grapl-common-test \
	grapl-analyzerlib-test \
	grapl-analyzer-executor-test \
	grapl-engagement-creator-test \
	grapl-engagement-edge-test \
	grapl-model-plugin-deployer-test \
	grapl-dgraph-ttl-test \
	grapl-notebook-test

JS_UNIT_TEST := \
	grapl-engagement-view-test \
	grapl-cdk-test

#
# Build
#

.PHONY: build
build: build-services ## alias for `services`

.PHONY: build-all
build-all: ## build all targets (incl. local, test, zip)
	$(DOCKER_BUILDX_BAKE) \
		 -f docker-compose.yml \
		 -f ./test/docker-compose.unit-tests.yml \
		 -f ./test/docker-compose.integration-tests.yml \
		 -f docker-compose.zips.yml

.PHONY: build-test-unit
build-test-unit:
	$(DOCKER_BUILDX_BAKE) -f ./test/docker-compose.unit-tests.yml

.PHONY: build-test-unit-rust
build-test-unit-rust:
	$(DOCKER_BUILDX_BAKE) -f ./test/docker-compose.unit-tests.yml \
		grapl-rust-test

.PHONY: build-test-unit-python
build-test-unit-python:
	$(DOCKER_BUILDX_BAKE) -f ./test/docker-compose.unit-tests.yml \
		$(PYTHON_UNIT_TESTS)

.PHONY: build-test-unit-js
build-test-unit-js:
	$(DOCKER_BUILDX_BAKE) -f ./test/docker-compose.unit-tests.yml \
		$(JS_UNIT_TEST)

.PHONY: build-test-typecheck
build-test-typecheck:
	docker buildx bake -f ./test/docker-compose.typecheck-tests.yml

.PHONY: build-test-integration
build-test-integration:
	$(DOCKER_BUILDX_BAKE) -f docker-compose.yml -f ./test/docker-compose.integration-tests.yml

.PHONY: build-test-e2e
build-test-e2e:
	$(DOCKER_BUILDX_BAKE) -f docker-compose.yml -f ./test/docker-compose.e2e-tests.yml

.PHONY: build-services
build-services: ## build Grapl services
	$(DOCKER_BUILDX_BAKE) -f docker-compose.yml

.PHONY: build-aws
build-aws: ## build services for Grapl in AWS
	$(DOCKER_BUILDX_BAKE) -f docker-compose.zips.yml

#
# Test
#

RUN_UNIT_TEST := test/docker-compose-with-error.sh \
	-f ./test/docker-compose.unit-tests.yml \
	-p grapl-unit_tests

.PHONY: test-unit
test-unit: build-test-unit ## build and run unit tests
	$(RUN_UNIT_TEST)

.PHONY: test-unit-rust
test-unit-rust: build-test-unit-rust ## build and run unit tests - Rust
	$(RUN_UNIT_TEST) grapl-rust-test

.PHONY: test-unit-python
test-unit-python: build-test-unit-python ## build and run unit tests - Python
	$(RUN_UNIT_TEST) $(PYTHON_UNIT_TESTS)

.PHONY: test-unit-js
test-unit-js: build-test-unit-js ## build and run unit tests - JavaScript
	$(RUN_UNIT_TEST) $(JS_UNIT_TEST)

.PHONY: test-typecheck
test-typecheck: build-test-typecheck ## build and run typecheck tests
	test/docker-compose-with-error.sh \
		-f ./test/docker-compose.typecheck-tests.yml \
		-p grapl-typecheck_tests

.PHONY: test-integration
test-integration: build-test-integration ## build and run integration tests
	docker-compose -f docker-compose.yml up --force-recreate -d
	# save exit code to allow for `make down` in event of test failure
	test/docker-compose-with-error.sh \
		-f ./test/docker-compose.integration-tests.yml \
		-p grapl-integration_tests; \
	EXIT_CODE=$$?; \
	$(MAKE) down; \
	exit $$EXIT_CODE

.PHONY: test-e2e
test-e2e: build-test-e2e ## build and run e2e tests
	docker-compose -f docker-compose.yml up --force-recreate -d
	# save exit code to allow for `make down` in event of test failure
	test/docker-compose-with-error.sh \
		-f ./test/docker-compose.e2e-tests.yml \
		-p grapl-e2e_tests; \
	EXIT_CODE=$$?; \
	$(MAKE) down; \
	exit $$EXIT_CODE

.PHONY: lint
lint: ## Run lint checks
	cd src/rust; cargo fmt -- --check
	./etc/ci_scripts/py_lint.sh --check-only

#
# else
#

.PHONY: clean
clean: ## Prune all docker build cache
	docker builder prune -a -f
	# Seems the docker service could use restarting every once in a while
	sudo service docker restart

.PHONY: clean-mount-cache
clean-mount-cache: ## Prune all docker mount cache
	docker builder prune --filter type=exec.cachemount

.PHONY: release
release: ## 'make zip' with cargo --release
	$(MAKE) PROFILE=release zip

.PHONY: zip
zip: build-aws ## Generate zips for use in AWS
	docker-compose -f docker-compose.zips.yml up

.PHONY: up
up: build-services ## build local services and docker-compose up
	docker-compose -f docker-compose.yml up

.PHONY: down
down: ## docker-compose down
	docker-compose -f docker-compose.yml down

.PHONY: help
help: ## print this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9_-]+:.*?## / {gsub("\\\\n",sprintf("\n%22c",""), $$2);printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
