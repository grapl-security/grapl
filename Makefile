#
# Makefile for developing using Docker
#

.DEFAULT_GOAL := help

-include .env
TAG ?= latest
RUST_BUILD ?= debug
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
	--file docker-compose.lambda-zips.rust.yml

export EVERY_COMPOSE_FILE=--file docker-compose.yml \
	--file ./test/docker-compose.unit-tests-rust.yml \
	--file ./test/docker-compose.unit-tests-js.yml \
	--file ./test/docker-compose.integration-tests.yml \
	--file ./test/docker-compose.e2e-tests.yml \
	--file ./test/docker-compose.typecheck-tests.yml \
        --file ./test/docker-compose.graplctl.yml \
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


##@ Build 🔨

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
build-test-typecheck: build-python-wheels
	docker buildx bake --file ./test/docker-compose.typecheck-tests.yml

.PHONY: build-test-integration
build-test-integration: build-services
	$(WITH_LOCAL_GRAPL_ENV) \
	$(DOCKER_BUILDX_BAKE) --file ./test/docker-compose.integration-tests.yml

.PHONY: build-test-e2e
build-test-e2e: build-services
	$(WITH_LOCAL_GRAPL_ENV) \
	$(DOCKER_BUILDX_BAKE) --file ./test/docker-compose.e2e-tests.yml

.PHONY: build-python-wheels
build-python-wheels:  ## Build all Python wheels
	./pants filter --target-type=python_distribution :: | xargs ./pants package

.PHONY: build-services
build-services: graplctl lambdas build-python-wheels ## Build Grapl services
	$(DOCKER_BUILDX_BAKE) --file docker-compose.build.yml

.PHONY: build-formatter
build-formatter:
	$(DOCKER_BUILDX_BAKE) \
		--file ./docker-compose.formatter.yml

.PHONY: graplctl
graplctl: ## Build graplctl and install it to the project root
	./pants package ./src/python/graplctl/graplctl
	cp ./dist/src.python.graplctl.graplctl/graplctl.pex ./bin/graplctl
	printf -- '\n${FMT_BOLD}graplctl${FMT_END} written to ${FMT_BLUE}./bin/graplctl${FMT_END}\n'
	$(DOCKER_BUILDX_BAKE) --file ./test/docker-compose.graplctl.yml

.PHONY: build-ux
build-ux: ## Build website assets
	cd src/js/engagement_view && yarn install && yarn build

.PHONY: lambdas
lambdas: lambdas-rust lambdas-js lambdas-python ## Generate all lambda zip files

.PHONY: lambdas-rust
lambdas-rust: ## Build Rust lambda zips
	$(DOCKER_BUILDX_BAKE) -f docker-compose.lambda-zips.rust.yml
	docker-compose -f docker-compose.lambda-zips.rust.yml up

.PHONY: lambdas-js
lambdas-js: ## Build JS lambda zips
	$(DOCKER_BUILDX_BAKE) -f docker-compose.lambda-zips.js.yml
	docker-compose -f docker-compose.lambda-zips.js.yml up

.PHONY: lambdas-python
lambdas-python: ## Build Python lambda zips
	./pants filter --target-type=python_awslambda :: | xargs ./pants package

##@ Test 🧪

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
# If you need to `pdb` these tests, add a `--debug` between `test` and `::`
test-unit-python: ## Run Python unit tests under Pants
	./pants --tag="-needs_work" test :: --pytest-args="-m \"not integration_test\""
# TODO: split this up so it uses a `./pants filter` to choose python tests and shell tests respectively

.PHONY: test-unit-js
test-unit-js: export COMPOSE_PROJECT_NAME := grapl-test-unit-js
test-unit-js: export COMPOSE_FILE := ./test/docker-compose.unit-tests-js.yml
test-unit-js: build-test-unit-js ## Build and run unit tests - JavaScript only
	test/docker-compose-with-error.sh

.PHONY: test-typecheck-docker
test-typecheck-docker: export COMPOSE_PROJECT_NAME := grapl-typecheck_tests
test-typecheck-docker: export COMPOSE_FILE := ./test/docker-compose.typecheck-tests.yml
test-typecheck-docker: build-test-typecheck ## Build and run typecheck tests (non-Pants)
	test/docker-compose-with-error.sh

.PHONY: test-typecheck-pants
test-typecheck-pants: ## Typecheck Python code with Pants
	./pants typecheck ::

.PHONY: test-typecheck
test-typecheck: test-typecheck-docker test-typecheck-pants ## Typecheck all Python Code

.PHONY: test-integration
test-integration: export COMPOSE_PROJECT_NAME := $(COMPOSE_PROJECT_INTEGRATION_TESTS)
test-integration: export COMPOSE_FILE := ./test/docker-compose.integration-tests.yml
test-integration: build-test-integration ## Build and run integration tests
	$(MAKE) test-with-env

.PHONY: test-e2e
test-e2e: export COMPOSE_PROJECT_NAME := $(COMPOSE_PROJECT_E2E_TESTS)
test-e2e: export export COMPOSE_FILE := ./test/docker-compose.e2e-tests.yml
test-e2e: build-test-e2e ## Build and run e2e tests
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
		etc/ci_scripts/dump_artifacts.py --compose-project=${COMPOSE_PROJECT_NAME}
		$(MAKE) down
	}
	# Ensure we call stop even after test failure, and return exit code from
	# the test, not the stop command.
	trap stopGrapl EXIT
	$(WITH_LOCAL_GRAPL_ENV)
	# Bring up the Grapl environment and detach
	$(MAKE) up-detach
	# Run tests and check exit codes from each test container
	test/docker-compose-with-error.sh

##@ Lint 🧹

.PHONY: lint-rust
lint-rust: ## Run Rust lint checks
	cd src/rust; bin/format --check; bin/lint

.PHONY: lint-python
lint-python: ## Run Python lint checks
	./pants filter --target-type=python_library,python_tests :: | xargs ./pants lint

.PHONY: lint-shell
lint-shell: ## Run Shell lint checks
	./pants filter --target-type=shell_library :: | xargs ./pants lint

.PHONY: lint-prettier
lint-prettier: build-formatter ## Run ts/js/yaml lint checks
	docker-compose -f docker-compose.formatter.yml up lint-prettier

.PHONY: lint-packer
lint-packer: ## Check to see if Packer templates are formatted properly
	.buildkite/scripts/lint_packer.sh

.PHONY: lint
lint: lint-python lint-prettier lint-rust lint-shell lint-packer ## Run all lint checks

##@ Formatting 💅

.PHONY: format-rust
format-rust: ## Reformat all Rust code
	cd src/rust; bin/format --update

.PHONY: format-python
format-python: ## Reformat all Python code
	./pants fmt ::

.PHONY: format-prettier
format-prettier: build-formatter ## Reformat js/ts/yaml
	docker-compose -f docker-compose.formatter.yml up format-prettier

.PHONY: format-packer
format-packer: ## Reformat all Packer HCLs
	packer fmt -recursive packer/

.PHONY: format
format: format-python format-prettier format-rust format-packer ## Reformat all code

.PHONY: package-python-libs
package-python-libs: ## Create Python distributions for public libraries
	./pants filter --target-type=python_distribution :: | xargs ./pants package

##@ Local Grapl 💻

.PHONY: up
up: export COMPOSE_PROJECT_NAME="grapl"
up: build-services ## Build Grapl services and launch docker-compose up
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
		up --detach --force-recreate --always-recreate-deps

.PHONY: down
down: ## docker-compose down - both stops and removes the containers
	$(WITH_LOCAL_GRAPL_ENV)
	# This is only for killing the lambda containers that Localstack
	# spins up in our network, but that docker-compose doesn't know
	# about. This must be the network that is used in Localstack's
	# LAMBDA_DOCKER_NETWORK environment variable.
	-docker kill $(shell docker ps --quiet --filter=network=grapl-network)
	docker-compose $(EVERY_COMPOSE_FILE) down --timeout=0
	docker-compose $(EVERY_COMPOSE_FILE) --project-name $(COMPOSE_PROJECT_INTEGRATION_TESTS) down --timeout=0
	docker-compose $(EVERY_COMPOSE_FILE) --project-name $(COMPOSE_PROJECT_E2E_TESTS) down --timeout=0

.PHONY: stop
stop: ## docker-compose stop - stops (but preserves) the containers
	$(WITH_LOCAL_GRAPL_ENV)
	docker-compose $(EVERY_COMPOSE_FILE) stop

##@ Utility ⚙

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

.PHONY: clean-artifacts
clean-artifacts: ## Remove all dumped artifacts from test runs (see dump_artifacts.py)
	rm -Rf test_artifacts

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

.PHONY: repl
repl: ## Run an interactive ipython repl that can import from grapl-common etc
	./pants --no-pantsd repl --shell=ipython src/python/repl

.PHONY: pulumi-prep
pulumi-prep: graplctl lambdas build-ux ## Prepare some artifacts in advance of running a Pulumi update (does not run Pulumi!)

.PHONY: update-shared
update-buildkite-shared: ## Pull in changes from grapl-security/buildkite-common
	git subtree pull --prefix .buildkite/shared git@github.com:grapl-security/buildkite-common.git main --squash
