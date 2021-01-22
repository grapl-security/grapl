#
# Makefile for developing using Docker
#

-include .env
TAG ?= latest
PROFILE ?= debug
UID = $(shell id -u)
GID = $(shell id -g)
DOCKER_BUILDX_BAKE_OPTS ?=
ifneq ($(GRAPL_RUST_ENV_FILE),)
DOCKER_BUILDX_BAKE_OPTS += --set *.secrets=id=rust_env,src="$(GRAPL_RUST_ENV_FILE)"
endif
export

export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

DOCKER_BUILDX_BAKE := docker buildx bake $(DOCKER_BUILDX_BAKE_OPTS)

#
# Build
#

.PHONY: build
build: build-services ## Alias for `services` (default)

.PHONY: build-all
build-all: ## Build all targets (incl. services, tests, zip)
	$(DOCKER_BUILDX_BAKE) \
		-f docker-compose.yml \
		-f ./test/docker-compose.unit-tests-rust.yml \
		-f ./test/docker-compose.unit-tests-python.yml \
		-f ./test/docker-compose.unit-tests-js.yml \
		-f ./test/docker-compose.integration-tests.yml \
		-f ./test/docker-compose.e2e-tests.yml \
		-f ./test/docker-compose.typecheck-tests.yml \
		-f docker-compose.zips.yml

.PHONY: build-test-unit
build-test-unit:
	$(DOCKER_BUILDX_BAKE) \
		-f ./test/docker-compose.unit-tests-rust.yml \
		-f ./test/docker-compose.unit-tests-python.yml \
		-f ./test/docker-compose.unit-tests-js.yml

.PHONY: build-test-unit-rust
build-test-unit-rust:
	$(DOCKER_BUILDX_BAKE) \
		-f ./test/docker-compose.unit-tests-rust.yml

.PHONY: build-test-unit-python
build-test-unit-python:
	$(DOCKER_BUILDX_BAKE) \
		-f ./test/docker-compose.unit-tests-python.yml

.PHONY: build-test-unit-js
build-test-unit-js:
	$(DOCKER_BUILDX_BAKE) \
		-f ./test/docker-compose.unit-tests-js.yml

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
build-services: ## Build Grapl services
	$(DOCKER_BUILDX_BAKE) -f docker-compose.yml

.PHONY: build-aws
build-aws: ## Build services for Grapl in AWS (subset of all services)
	$(DOCKER_BUILDX_BAKE) -f docker-compose.zips.yml

#
# Test
#

.PHONY: test-unit
test-unit: build-test-unit ## Build and run unit tests
	test/docker-compose-with-error.sh \
		-p grapl-test-unit \
		-f ./test/docker-compose.unit-tests-rust.yml \
		-f ./test/docker-compose.unit-tests-python.yml \
		-f ./test/docker-compose.unit-tests-js.yml

.PHONY: test-unit-rust
test-unit-rust: build-test-unit-rust ## Build and run unit tests - Rust only
	test/docker-compose-with-error.sh \
		-p grapl-test-unit-rust \
		-f ./test/docker-compose.unit-tests-rust.yml

.PHONY: test-unit-python
test-unit-python: build-test-unit-python ## Build and run unit tests - Python only
	test/docker-compose-with-error.sh \
		-p grapl-test-unit-python \
		-f ./test/docker-compose.unit-tests-python.yml

.PHONY: test-unit-js
test-unit-js: build-test-unit-js ## Build and run unit tests - JavaScript only
	test/docker-compose-with-error.sh \
		-p grapl-test-unit-js \
		-f ./test/docker-compose.unit-tests-js.yml

.PHONY: test-typecheck
test-typecheck: build-test-typecheck ## Build and run typecheck tests
	test/docker-compose-with-error.sh \
		-f ./test/docker-compose.typecheck-tests.yml \
		-p grapl-typecheck_tests

.PHONY: test-integration
test-integration: build-test-integration ## Build and run integration tests
	docker-compose -f docker-compose.yml up --force-recreate -d
	# save exit code to allow for `make down` in event of test failure
	test/docker-compose-with-error.sh \
		-f ./test/docker-compose.integration-tests.yml \
		-p grapl-integration_tests; \
	EXIT_CODE=$$?; \
	$(MAKE) down; \
	exit $$EXIT_CODE

.PHONY: test-e2e
test-e2e: build-test-e2e ## Build and run e2e tests
	docker-compose -f docker-compose.yml up --force-recreate -d
	# save exit code to allow for `make down` in event of test failure
	test/docker-compose-with-error.sh \
		-f ./test/docker-compose.e2e-tests.yml \
		-p grapl-e2e_tests; \
	EXIT_CODE=$$?; \
	$(MAKE) down; \
	exit $$EXIT_CODE

.PHONY: test
test: test-unit test-integration test-e2e test-typecheck ## Run all tests

.PHONY: lint-rust
lint-rust: ## Run Rust lint checks
	cd src/rust; cargo fmt -- --check

.PHONY: lint-python
lint-python: ## Run Python lint checks
	./etc/ci_scripts/py_lint.sh --check-only

.PHONY: lint
lint: lint-rust lint-python ## Run all lint checks


#
# else
#

.PHONY: clean
clean: ## Prune all docker build cache
	docker builder prune -a -f
	# Seems the docker service could use restarting every once in a while
	sudo service docker restart

.PHONY: clean-mount-cache
clean-mount-cache: ## Prune all docker mount cache (used by sccache)
	docker builder prune --filter type=exec.cachemount

.PHONY: release
release: ## 'make build-services' with cargo --release
	$(MAKE) PROFILE=release build-services

.PHONY: zip
zip: build-aws ## Generate zips for deploying to AWS (src/js/grapl-cdk/zips/)
	docker-compose -f docker-compose.zips.yml up

.PHONY: up
up: build-services ## Build Grapl services and launch docker-compose up
	docker-compose -f docker-compose.yml up

.PHONY: down
down: ## docker-compose down
	docker-compose -f docker-compose.yml down

.PHONY: restart
down: ## docker-compose down
	docker-compose -f docker-compose.Makefile.yml down

.PHONY: help
help: ## Print this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9_-]+:.*?## / {gsub("\\\\n",sprintf("\n%22c",""), $$2);printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
