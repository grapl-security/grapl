#
# Makefile for developing using Docker
#

.DEFAULT_GOAL := help

-include .env
TAG ?= latest
CARGO_PROFILE ?= debug
UID = $(shell id -u)
GID = $(shell id -g)
DOCKER_BUILDX_BAKE_OPTS ?=
ifneq ($(GRAPL_RUST_ENV_FILE),)
DOCKER_BUILDX_BAKE_OPTS += --set *.secrets=id=rust_env,src="$(GRAPL_RUST_ENV_FILE)"
endif
COMPOSE_IGNORE_ORPHANS=1
COMPOSE_PROJECT_NAME ?= grapl
export

export EVERY_LAMBDA_COMPOSE_FILE=--file docker-compose.lambda-zips.js.yml \
	--file docker-compose.lambda-zips.python.yml \
	--file docker-compose.lambda-zips.rust.yml

export EVERY_COMPOSE_FILE=--file docker-compose.yml \
	--file ./test/docker-compose.unit-tests-rust.yml \
	--file ./test/docker-compose.unit-tests-js.yml \
	--file ./test/docker-compose.integration-tests.yml \
	--file ./test/docker-compose.e2e-tests.yml \
	--file ./test/docker-compose.typecheck-tests.yml \
	${EVERY_LAMBDA_COMPOSE_FILE}

DOCKER_BUILDX_BAKE := docker buildx bake $(DOCKER_BUILDX_BAKE_OPTS)

COMPOSE_PROJECT_INTEGRATION_TESTS := grapl-integration_tests
COMPOSE_PROJECT_E2E_TESTS := grapl-e2e_tests


# Use a single shell for each of our targets, which allows us to use the 'trap'
# built-in in our targets. We set the 'errexit' shell option to preserve
# execution behavior, where failure from one line in a target will result in
# Make error.
# https://www.gnu.org/software/make/manual/html_node/One-Shell.html
SHELL := bash
.ONESHELL:
# errexit nounset noclobber
.SHELLFLAGS := \
-e \
-u \
-o pipefail \
-c

# Note: it doesn't seem to like a single-quote nested in a double-quote!
WITH_RETRY = ./build-support/retry.sh --sleep=0 --tries=3 --

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

FMT_BLUE = \033[36m
FMT_PURPLE = \033[35m
FMT_BOLD = \033[1m
FMT_END = \033[0m
VSC_DEBUGGER_DOCS_LINK = https://grapl.readthedocs.io/en/latest/debugging/vscode_debugger.html


.PHONY: help
help: ## Print this help
	@printf -- '\n'
	@printf -- '                                                     __ \n'
	@printf -- '             (≡)         ____ _ _____ ____ _ ____   / / \n'
	@printf -- '                \       / __ `// ___// __ `// __ \ / /  \n'
	@printf -- '                (≡)    / /_/ // /   / /_/ // /_/ // /   \n'
	@printf -- '                /      \__, //_/    \__,_// .___//_/    \n'
	@printf -- '             (≡)      /____/             /_/            \n'
	@printf -- '\n'
	@printf -- '${FMT_BOLD}Useful environment variables (with examples):${FMT_END}\n'
	@printf -- '  ${FMT_PURPLE}TARGETS${FMT_END}="typecheck-analyzer-executor typecheck-grapl-common" make test-typecheck\n'
	@printf -- '    to only run a subset of test targets.\n'
	@printf -- '\n'
	@printf -- '  ${FMT_PURPLE}KEEP_TEST_ENV=1${FMT_END} make test-integration\n'
	@printf -- '    to keep the test environment around after a test suite.\n'
	@printf -- '\n'
	@printf -- '  ${FMT_PURPLE}DEBUG_SERVICES${FMT_END}="graphql_endpoint grapl_e2e_tests" make test-e2e\n'
	@printf -- '    to launch the VSCode Debugger (see ${VSC_DEBUGGER_DOCS_LINK}).\n'
	@printf -- '\n'
	@printf -- '  ${FMT_BOLD}FUN FACT${FMT_END}: You can also specify these as postfix, like:\n'
	@printf -- '    make test-something KEEP_TEST_ENV=1\n'
	@printf '\n'
	@awk 'BEGIN {FS = ":.*##"; printf "Usage: make ${FMT_BLUE}<target>${FMT_END}\n"} \
		 /^[a-zA-Z0-9_-]+:.*?##/ { printf "  ${FMT_BLUE}%-46s${FMT_END} %s\n", $$1, $$2 } \
		 /^##@/ { printf "\n${FMT_BOLD}%s${FMT_END}\n", substr($$0, 5) } ' \
		 $(MAKEFILE_LIST)
	@printf '\n'


##@ Build

.PHONY: build
build: build-services ## Alias for `services` (default)

.PHONY: build-release
build-release: ## 'make build-services' with cargo --release
	$(MAKE) CARGO_PROFILE=release build-services

.PHONY: build-all
build-all: ## Build all targets (incl. services, tests, zip)
	$(WITH_LOCAL_GRAPL_ENV) $(DOCKER_BUILDX_BAKE) $(EVERY_COMPOSE_FILE)

.PHONY: build-test-unit
build-test-unit:
	$(DOCKER_BUILDX_BAKE) \
		--file ./test/docker-compose.unit-tests-rust.yml \
		--file ./test/docker-compose.unit-tests-js.yml

.PHONY: build-test-unit-rust
build-test-unit-rust:
	$(DOCKER_BUILDX_BAKE) \
		--file ./test/docker-compose.unit-tests-rust.yml

.PHONY: build-test-unit-js
build-test-unit-js:
	$(DOCKER_BUILDX_BAKE) \
		--file ./test/docker-compose.unit-tests-js.yml

.PHONY: build-test-typecheck
build-test-typecheck:
	docker buildx bake --file ./test/docker-compose.typecheck-tests.yml

.PHONY: build-test-integration
build-test-integration: build-services
	$(WITH_LOCAL_GRAPL_ENV) \
	$(DOCKER_BUILDX_BAKE) --file ./test/docker-compose.integration-tests.yml

.PHONY: build-test-e2e
build-test-e2e: build-services
	$(WITH_LOCAL_GRAPL_ENV) \
	$(DOCKER_BUILDX_BAKE) --file ./test/docker-compose.e2e-tests.yml

.PHONY: build-services
build-services: ## Build Grapl services
	$(DOCKER_BUILDX_BAKE) --file docker-compose.build.yml

.PHONY: build-lambdas
build-lambdas: ## Build services for Grapl in AWS (subset of all services)
	$(DOCKER_BUILDX_BAKE) $(EVERY_LAMBDA_COMPOSE_FILE)

.PHONY: graplctl
graplctl: ## Build graplctl and install it to the project root
	./pants package ./src/python/graplctl/graplctl
	cp ./dist/src.python.graplctl.graplctl/graplctl.pex ./bin/graplctl

##@ Test

.PHONY: test
test: test-unit test-integration test-e2e test-typecheck ## Run all tests

.PHONY: test-unit
test-unit: export COMPOSE_PROJECT_NAME := grapl-test-unit
test-unit: export COMPOSE_FILE := ./test/docker-compose.unit-tests-rust.yml:./test/docker-compose.unit-tests-js.yml
test-unit: build-test-unit test-unit-python ## Build and run unit tests
	test/docker-compose-with-error.sh

.PHONY: test-unit-rust
test-unit-rust: export COMPOSE_PROJECT_NAME := grapl-test-unit-rust
test-unit-rust: export COMPOSE_FILE := ./test/docker-compose.unit-tests-rust.yml
test-unit-rust: build-test-unit-rust ## Build and run unit tests - Rust only
	test/docker-compose-with-error.sh

.PHONY: test-unit-python
# Long term, it would be nice to organize the tests with Pants
# tags, rather than pytest tags
test-unit-python: ## Run Python unit tests under Pants
	$(WITH_RETRY) ./pants --tag="-needs_work" test :: --pytest-args="-m \"not integration_test\""

.PHONY: test-unit-js
test-unit-js: export COMPOSE_PROJECT_NAME := grapl-test-unit-js
test-unit-js: export COMPOSE_FILE := ./test/docker-compose.unit-tests-js.yml
test-unit-js: build-test-unit-js ## Build and run unit tests - JavaScript only
	test/docker-compose-with-error.sh

.PHONY: test-typecheck
test-typecheck: export COMPOSE_PROJECT_NAME := grapl-typecheck_tests
test-typecheck: export COMPOSE_FILE := ./test/docker-compose.typecheck-tests.yml
test-typecheck: build-test-typecheck ## Build and run typecheck tests (non-Pants)
	test/docker-compose-with-error.sh

.PHONY: test-typecheck-pulumi
test-typecheck-pulumi: ## Typecheck Pulumi Python code
	$(WITH_RETRY) ./pants typecheck pulumi::

.PHONY: test-typecheck-build-support
test-typecheck-build-support: ## Typecheck build-support Python code
	$(WITH_RETRY) ./pants typecheck build-support::

# Right now, we're only typechecking a select portion of code with
# Pants until CM fixes https://github.com/pantsbuild/pants/issues/11553
.PHONY: test-typecheck-pants
test-typecheck-pants: test-typecheck-pulumi test-typecheck-build-support ## Typecheck Python code with Pants

.PHONY: test-integration
test-integration: export COMPOSE_PROJECT_NAME := $(COMPOSE_PROJECT_INTEGRATION_TESTS)
test-integration: export COMPOSE_FILE := ./test/docker-compose.integration-tests.yml
test-integration: build-test-integration modern-lambdas ## Build and run integration tests
	$(MAKE) test-with-env

.PHONY: test-e2e
test-e2e: export COMPOSE_PROJECT_NAME := $(COMPOSE_PROJECT_E2E_TESTS)
test-e2e: export export COMPOSE_FILE := ./test/docker-compose.e2e-tests.yml
test-e2e: build-test-e2e modern-lambdas ## Build and run e2e tests
	$(MAKE) test-with-env

# This target is not intended to be used directly from the command line, it's
# intended for tests in docker-compose files that need the Grapl environment.
.PHONY: test-with-env
test-with-env: # (Do not include help text - not to be used directly)
	stopGrapl() {
		# skip if KEEP_TEST_ENV is set
		if [[ -z "${KEEP_TEST_ENV}" ]]; then
			echo "Tearing down test environment"
		else
			echo "Keeping test environment" && return 0
		fi
		# Unset COMPOSE_FILE to help ensure it will be ignored with use of --file
		unset COMPOSE_FILE
		docker-compose --file docker-compose.yml stop;
	}
	# Ensure we call stop even after test failure, and return exit code from
	# the test, not the stop command.
	trap stopGrapl EXIT
	$(WITH_LOCAL_GRAPL_ENV)
	# Bring up the Grapl environment and detach
	$(MAKE) up-detach
	# Run tests and check exit codes from each test container
	test/docker-compose-with-error.sh

##@ Lint

.PHONY: lint-rust
lint-rust: ## Run Rust lint checks
	cd src/rust; bin/format --check; bin/lint

.PHONY: lint-python
lint-python: ## Run Python lint checks
	./pants lint ::

.PHONY: lint-js
lint-js: ## Run js lint checks
	cd src/js; bin/format.sh --check

.PHONY: lint
lint: lint-python lint-js lint-rust ## Run all lint checks

##@ Formatting

.PHONY: format-rust
format-rust: ## Reformat all Rust code
	cd src/rust; bin/format --update

.PHONY: format-python
format-python: ## Reformat all Python code
	./pants fmt ::

.PHONY: format-js
format-js: ## Reformat all js/ts code
	cd src/js; bin/format.sh --update

.PHONY: format
format: format-python format-js format-rust ## Reformat all code

.PHONY: package-python-libs
package-python-libs: ## Create Python distributions for public libraries
	./pants filter --filter-target-type=python_distribution :: | xargs ./pants package

##@ Local Grapl

.PHONY: up
up: build-services modern-lambdas ## Build Grapl services and launch docker-compose up
	$(WITH_LOCAL_GRAPL_ENV)
	docker-compose -f docker-compose.yml up

.PHONY: up-detach
up-detach: build-services ## Bring up local Grapl and detach to return control to tty
	# Primarily used for bringing up an environment for integration testing.
	# For use with a project name consider setting COMPOSE_PROJECT_NAME env var
	# Usage: `make up-detach`
	$(WITH_LOCAL_GRAPL_ENV)
	# We use this target with COMPOSE_FILE being set pointing to other files.
	# Although it seems specifying the `--file` option overrides that, we'll
	# explicitly unset that here to avoid potential surprises.
	unset COMPOSE_FILE
	docker-compose \
		--file docker-compose.yml \
		up --detach --force-recreate

.PHONY: down
down: ## docker-compose down - both stops and removes the containers
	$(WITH_LOCAL_GRAPL_ENV)
	docker-compose $(EVERY_COMPOSE_FILE) down --timeout=0
	docker-compose $(EVERY_COMPOSE_FILE) --project-name $(COMPOSE_PROJECT_INTEGRATION_TESTS) down --timeout=0
	docker-compose $(EVERY_COMPOSE_FILE) --project-name $(COMPOSE_PROJECT_E2E_TESTS) down --timeout=0

.PHONY: stop
stop: ## docker-compose stop - stops (but preserves) the containers
	$(WITH_LOCAL_GRAPL_ENV)
	docker-compose $(EVERY_COMPOSE_FILE) stop

##@ Utility

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

.PHONY: zip
zip: build-lambdas ## Generate zips for deploying to AWS (src/js/grapl-cdk/zips/)
	docker-compose $(EVERY_LAMBDA_COMPOSE_FILE) up
	$(MAKE) zip-pants

.PHONY: zip-pants
zip-pants: ## Generate Lambda zip artifacts using pants
	./pants filter --filter-target-type=python_awslambda :: | xargs ./pants package
	cp ./dist/src.python.provisioner.src/lambda.zip ./src/js/grapl-cdk/zips/provisioner-$(TAG).zip
	cp ./dist/src.python.engagement-creator/engagement-creator.zip ./src/js/grapl-cdk/zips/engagement-creator-$(TAG).zip
	cp ./dist/src.python.grapl-dgraph-ttl/lambda.zip ./src/js/grapl-cdk/zips/dgraph-ttl-$(TAG).zip

# This target is intended to help ease the transition to Pulumi, and
# using lambdas in local Grapl testing deployments. Essentially, every
# lambda that is deployed by Pulumi should be built here. Once
# everything is migrated to Pulumi, we can consolidate this target
# with other zip-generating targets
modern-lambdas: ## Generate lambda zips that are used in local Grapl and Pulumi deployments
	$(DOCKER_BUILDX_BAKE) -f docker-compose.lambda-zips.rust.yml
	docker-compose -f docker-compose.lambda-zips.rust.yml up
	$(MAKE) zip-pants

.PHONY: push
push: ## Push Grapl containers to Docker Hub
	docker-compose --file=docker-compose.build.yml push

.PHONY: e2e-logs
e2e-logs: ## All docker-compose logs
	$(WITH_LOCAL_GRAPL_ENV)
	docker-compose $(EVERY_COMPOSE_FILE) --project-name $(COMPOSE_PROJECT_E2E_TESTS) logs -f

.PHONY: docker-kill-all
docker-kill-all:  # Kill all currently running Docker containers
	docker kill $$(docker ps -aq)

.PHONY: populate-venv
populate-venv: ## Set up a Python virtualenv (you'll have to activate manually!)
	build-support/manage_virtualenv.sh populate
