#
# Makefile for developing using Docker
#

-include .env
TAG ?= latest
CARGO_PROFILE ?= debug
UID = $(shell id -u)
GID = $(shell id -g)
DOCKER_BUILDX_BAKE_OPTS ?=
ifneq ($(GRAPL_RUST_ENV_FILE),)
DOCKER_BUILDX_BAKE_OPTS += --set *.secrets=id=rust_env,src="$(GRAPL_RUST_ENV_FILE)"
endif
export

export EVERY_COMPOSE_FILE=-f docker-compose.yml \
	-f ./test/docker-compose.unit-tests-rust.yml \
	-f ./test/docker-compose.unit-tests-js.yml \
	-f ./test/docker-compose.integration-tests.yml \
	-f ./test/docker-compose.e2e-tests.yml \
	-f ./test/docker-compose.typecheck-tests.yml \
	-f docker-compose.zips.yml

DOCKER_BUILDX_BAKE := docker buildx bake $(DOCKER_BUILDX_BAKE_OPTS)

# Our `docker-compose.yml` file declares the setup of a "local Grapl"
# environment, which can be used to locally exercise a Grapl system,
# either manually or through automated integration and end-to-end
# ("e2e") tests. Because this environment requires a large amount of
# configuration data, which must also be shared between several
# different files (including, but not limited to, the aforementioned
# testing environments), this information has been extracted into an
# environment file for reuse.
#
# Currently, however, `docker buildx` recognizes `.env` files, but NOT
# `--env-file` options, like `docker-compose` does. This means that it
# is rather tricky to share environment variables across both tools in
# a general and explicit way, while also preserving the ability for
# users to use an `.env` file in the repo root for individual
# customizations.
#
# To try and balance these concerns of compatibility, explicitness,
# and flexibility, we'll use this snippet to establish an environment
# for subsequent commands in a Makefile target to run in. Any `docker
# buildx` or `docker-compose` calls that require this particular
# environment should place this in front of it.
#
# e.g., $(WITH_LOCAL_GRAPL_ENV) docker-compose -f docker-compose.yml up
#
# Currently, any command that directly uses or depends on the
# `docker-compose.yml` file should use this. (Recall that each line of
# a recipe runs in its own subshell, to keep that in mind if you have
# multiple commands that need this environment.)
#
# The user's original calling environment will not polluted in any
# way.
WITH_LOCAL_GRAPL_ENV := set -o allexport; . ./local-grapl.env; set +o allexport;

#
# Build
#

.PHONY: build
build: build-services ## Alias for `services` (default)

.PHONY: build-all
build-all: ## Build all targets (incl. services, tests, zip)
	$(WITH_LOCAL_GRAPL_ENV) $(DOCKER_BUILDX_BAKE) $(EVERY_COMPOSE_FILE)

.PHONY: build-test-unit
build-test-unit:
	$(DOCKER_BUILDX_BAKE) \
		-f ./test/docker-compose.unit-tests-rust.yml \
		-f ./test/docker-compose.unit-tests-js.yml

.PHONY: build-test-unit-rust
build-test-unit-rust:
	$(DOCKER_BUILDX_BAKE) \
		-f ./test/docker-compose.unit-tests-rust.yml

.PHONY: build-test-unit-js
build-test-unit-js:
	$(DOCKER_BUILDX_BAKE) \
		-f ./test/docker-compose.unit-tests-js.yml

.PHONY: build-test-typecheck
build-test-typecheck:
	docker buildx bake -f ./test/docker-compose.typecheck-tests.yml

.PHONY: build-test-integration
build-test-integration: build-services
	$(WITH_LOCAL_GRAPL_ENV) \
	$(DOCKER_BUILDX_BAKE) -f ./test/docker-compose.integration-tests.yml

.PHONY: build-test-e2e
build-test-e2e: build-services
	$(WITH_LOCAL_GRAPL_ENV) \
	$(DOCKER_BUILDX_BAKE) -f ./test/docker-compose.e2e-tests.yml

.PHONY: build-services
build-services: ## Build Grapl services
	$(DOCKER_BUILDX_BAKE) -f docker-compose.build.yml

.PHONY: build-aws
build-aws: ## Build services for Grapl in AWS (subset of all services)
	$(DOCKER_BUILDX_BAKE) -f docker-compose.zips.yml

#
# Test
#

.PHONY: test-unit
test-unit: build-test-unit test-unit-python ## Build and run unit tests
	test/docker-compose-with-error.sh \
		-p grapl-test-unit \
		-f ./test/docker-compose.unit-tests-rust.yml \
		-f ./test/docker-compose.unit-tests-js.yml

.PHONY: test-unit-rust
test-unit-rust: build-test-unit-rust ## Build and run unit tests - Rust only
	test/docker-compose-with-error.sh \
		-p grapl-test-unit-rust \
		-f ./test/docker-compose.unit-tests-rust.yml

.PHONY: test-unit-python
# Long term, it would be nice to organize the tests with Pants
# tags, rather than pytest tags
test-unit-python: ## Run Python unit tests under Pants
	./pants --tag="-needs_work" test :: --pytest-args="-m 'not integration_test'"

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
test-integration: export COMPOSE_IGNORE_ORPHANS=1
test-integration: build-test-integration ## Build and run integration tests
	$(WITH_LOCAL_GRAPL_ENV) \
	$(MAKE) up-detach PROJECT_NAME="grapl-integration_tests" && \
	test/docker-compose-with-error.sh \
		-f ./test/docker-compose.integration-tests.yml \
		-p "grapl-integration_tests"

.PHONY: test-e2e
test-e2e: export COMPOSE_IGNORE_ORPHANS=1
test-e2e: build-test-e2e ## Build and run e2e tests
	$(WITH_LOCAL_GRAPL_ENV) \
	$(MAKE) up-detach PROJECT_NAME="grapl-e2e_tests" && \
	test/docker-compose-with-error.sh \
		-f ./test/docker-compose.e2e-tests.yml \
		-p "grapl-e2e_tests"

.PHONY: test
test: test-unit test-integration test-e2e test-typecheck ## Run all tests

.PHONY: lint-rust
lint-rust: ## Run Rust lint checks
	cd src/rust; bin/format --check; bin/lint

.PHONY: lint-python
lint-python: ## Run Python lint checks
	./pants lint ::

.PHONY: lint
lint: lint-rust lint-python ## Run all lint checks

.PHONY: format-rust
format-rust: ## Reformat all Rust code
	cd src/rust; bin/format --update

.PHONY: format-python
format-python: ## Reformat all Python code
	./pants fmt ::

.PHONY: format
format: format-rust format-python ## Reformat all code

.PHONY: package-python-libs
package-python-libs: ## Create Python distributions for public libraries
	./pants filter --filter-target-type=python_distribution :: | xargs ./pants package

#
# else
#

.PHONY: clean
clean: ## Prune all docker build cache and remove Grapl containers and images
	docker builder prune --all --force
	# Remove all Grapl containers - continue on error (no containers found)
	docker rm --volumes --force $$(docker ps --filter "name=grapl*" --all --quiet) 2>/dev/null || true
	# Remove all Grapl images = continue on error (no images found)
	docker rmi --force $$(docker images --filter reference="grapl/*" --quiet) 2>/dev/null || true

.PHONY: clean-mount-cache
clean-mount-cache: ## Prune all docker mount cache (used by sccache)
	docker builder prune --filter type=exec.cachemount

.PHONY: release
release: ## 'make build-services' with cargo --release
	$(MAKE) CARGO_PROFILE=release build-services

.PHONY: zip
zip: build-aws ## Generate zips for deploying to AWS (src/js/grapl-cdk/zips/)
	docker-compose -f docker-compose.zips.yml up

.PHONY: deploy
deploy: zip ## CDK deploy to AWS
	src/js/grapl-cdk/deploy_all.sh

.PHONY: push
push: ## Push Grapl containers to Docker Hub
	docker-compose --file=docker-compose.build.yml push

.PHONY: up
up: build-services ## Build Grapl services and launch docker-compose up
	$(WITH_LOCAL_GRAPL_ENV) \
	docker-compose -f docker-compose.yml up

.PHONY: up-detach
up-detach: build-services ## Docker-compose up + detach and return control to tty
	# Primarily used for bringing up an environment for integration testing.
	# Usage: `make up-detach PROJECT_NAME=asdf`
	$(WITH_LOCAL_GRAPL_ENV) \
	docker-compose \
		-p $(PROJECT_NAME) \
		-f docker-compose.yml \
		up --detach --force-recreate

.PHONY: down
down: ## docker-compose down - both stops and removes the containers
	$(WITH_LOCAL_GRAPL_ENV) \
	docker-compose $(EVERY_COMPOSE_FILE) down --remove-orphans

.PHONY: stop
stop: ## docker-compose stop - stops (but preserves) the containers
	$(WITH_LOCAL_GRAPL_ENV) \
	docker-compose $(EVERY_COMPOSE_FILE) stop

.PHONY: e2e-logs
e2e-logs: ## All docker-compose logs
	$(WITH_LOCAL_GRAPL_ENV) \
	docker-compose $(EVERY_COMPOSE_FILE) -p grapl-e2e_tests logs -f

.PHONY: help
help: ## Print this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9_-]+:.*?## / {gsub("\\\\n",sprintf("\n%22c",""), $$2);printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: docker-kill-all
docker-kill-all:  # Kill all currently running Docker containers
	docker kill $$(docker ps -aq)
