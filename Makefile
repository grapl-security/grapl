#
# Makefile for developing using Docker
#

.DEFAULT_GOAL := help

-include .env
TAG ?= dev
RUST_BUILD ?= debug
USERNAME = $(shell id -u -n)
UID = $(shell id -u)
GID = $(shell id -g)
PWD = $(shell pwd)
COMPOSE_USER=${UID}:${GID}
DOCKER_BUILDX_BAKE_OPTS ?=
ifneq ($(GRAPL_RUST_ENV_FILE),)
DOCKER_BUILDX_BAKE_OPTS += --set *.secrets=id=rust_env,src="$(GRAPL_RUST_ENV_FILE)"
endif
COMPOSE_IGNORE_ORPHANS=1
COMPOSE_PROJECT_NAME ?= grapl
export

export EVERY_COMPOSE_FILE=--file docker-compose.yml \
	--file ./test/docker-compose.unit-tests-rust.yml \
	--file ./test/docker-compose.unit-tests-js.yml \
	--file ./test/docker-compose.integration-tests.build.yml \
	--file ./test/docker-compose.e2e-tests.build.yml \

DOCKER_BUILDX_BAKE := docker buildx bake $(DOCKER_BUILDX_BAKE_OPTS)

COMPOSE_PROJECT_INTEGRATION_TESTS := grapl-integration_tests
COMPOSE_PROJECT_E2E_TESTS := grapl-e2e_tests

# All the services defined in the docker-compose.check.yml file are
# run with the same general arguments; just supply the service name to
# run.
#
# While we would ultimately like to run all these containers as a
# non-root user, some currently seem to require that; to accommodate
# all such images, we provide two helpful macros.
DOCKER_COMPOSE_CHECK := docker-compose --file=docker-compose.check.yml run --rm
NONROOT_DOCKER_COMPOSE_CHECK := ${DOCKER_COMPOSE_CHECK} --user=${COMPOSE_USER}

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

.PHONY: build-service-pexs
build-service-pexs:
	./pants package ./src/python/analyzer_executor/src
	./pants package ./src/python/engagement-creator/engagement_creator:pex

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
	$(DOCKER_BUILDX_BAKE) \
		--file ./test/docker-compose.typecheck-tests.yml

.PHONY: build-test-integration
build-test-integration: build-local
	$(WITH_LOCAL_GRAPL_ENV) \
	$(DOCKER_BUILDX_BAKE) --file ./test/docker-compose.integration-tests.build.yml

.PHONY: build-test-e2e
build-test-e2e: build
	$(WITH_LOCAL_GRAPL_ENV) \
	$(DOCKER_BUILDX_BAKE) --file ./test/docker-compose.e2e-tests.build.yml

.PHONY: build-lambda-zips
build-lambda-zips: build-lambda-zips-rust build-lambda-zips-js build-lambda-zips-python build-service-pexs ## Generate all lambda zip files

.PHONY: build-lambda-zips-rust
build-lambda-zips-rust: ## Build Rust lambda zips
	$(DOCKER_BUILDX_BAKE) \
		--file docker-compose.lambda-zips.rust.yml
	# Extract the zip from the Docker image.
	# Rely on the default CMD for copying artifact to /dist mount point.
	docker-compose \
		--file docker-compose.lambda-zips.rust.yml \
		run \
		--rm \
		--user "${UID}:${GID}" \
		--volume="${PWD}/dist":/dist \
		metric-forwarder-zip

.PHONY: build-lambda-zips-js
build-lambda-zips-js: ## Build JS lambda zips
	$(DOCKER_BUILDX_BAKE) \
		--file docker-compose.lambda-zips.js.yml
	# Extract the zip from the Docker image.
	# Rely on the default CMD for copying artifact to /dist mount point.
	docker-compose \
		--file docker-compose.lambda-zips.js.yml \
		run \
		--rm \
		--user "${UID}:${GID}" \
		--volume="${PWD}/dist":/dist \
		graphql-endpoint-zip

.PHONY: build-lambda-zips-python
build-lambda-zips-python: build-python-wheels ## Build Python lambda zips
	./pants filter --target-type=python_awslambda :: | xargs ./pants package

.PHONY: build-python-wheels
build-python-wheels:  ## Build all Python wheels
	./pants filter --target-type=python_distribution :: | xargs ./pants package

.PHONY: build-docker-images-local
build-docker-images-local:
	$(WITH_LOCAL_GRAPL_ENV) \
	$(MAKE) build-docker-images

.PHONY: build-docker-images
build-docker-images: graplctl build-ux
	$(DOCKER_BUILDX_BAKE) --file docker-compose.build.yml

.PHONY: build
build: build-lambda-zips build-docker-images ## Build Grapl services

.PHONY: build-local
build-local: build-lambda-zips build-docker-images-local ## Build Grapl services

.PHONY: build-formatter
build-formatter:
	$(DOCKER_BUILDX_BAKE) \
		--file ./docker-compose.formatter.yml

.PHONY: graplctl
graplctl: ## Build graplctl and install it to ./bin
	./pants package ./src/python/graplctl/graplctl
	cp ./dist/src.python.graplctl.graplctl/graplctl.pex ./bin/graplctl
	printf -- '\n${FMT_BOLD}graplctl${FMT_END} written to ${FMT_BLUE}./bin/graplctl${FMT_END}\n'

.PHONY: grapl-template-generator
grapl-template-generator: ## Build the Grapl Template Generator and install it to ./bin
	./pants package ./src/python/grapl-template-generator/grapl_template_generator
	cp \
		./dist/src.python.grapl-template-generator.grapl_template_generator/grapl_template_generator.pex \
		./bin/grapl-template-generator
	printf -- '\n${FMT_BOLD}Template Generator${FMT_END} written to ${FMT_BLUE}./bin/grapl-template-generator${FMT_END}\n'

.PHONY: dump-artifacts
dump-artifacts:  # Run the script that dumps Nomad/Docker logs after test runs
	./pants run ./etc/ci_scripts/dump_artifacts --run-args="--compose-project=${COMPOSE_PROJECT_NAME}"

.PHONY: build-ux
build-ux: ## Build website assets
	$(MAKE) -C src/js/engagement_view build
	cp -r \
		"${PWD}/src/js/engagement_view/build/." \
		"${PWD}/src/rust/grapl-web-ui/frontend/"

##@ Test 🧪

.PHONY: test-unit
test-unit: export COMPOSE_PROJECT_NAME := grapl-test-unit
test-unit: export COMPOSE_FILE := ./test/docker-compose.unit-tests-rust.yml:./test/docker-compose.unit-tests-js.yml
test-unit: build-test-unit test-unit-python test-unit-shell ## Build and run unit tests
	test/docker-compose-with-error.sh

.PHONY: test-unit-rust
test-unit-rust: export COMPOSE_PROJECT_NAME := grapl-test-unit-rust
test-unit-rust: export COMPOSE_FILE := ./test/docker-compose.unit-tests-rust.yml
test-unit-rust: build-test-unit-rust ## Build and run unit tests - Rust only
	test/docker-compose-with-error.sh

.PHONY: test-unit-python
# Long term, it would be nice to organize the tests with Pants
# tags, rather than pytest tags
# If you need to `pdb` these tests, add a `--debug` after `./pants test`
test-unit-python: ## Run Python unit tests under Pants
	./pants filter --filter-target-type="python_tests" :: \
	| xargs ./pants --tag="-needs_work" test --pytest-args="-m \"not integration_test\""


.PHONY: test-unit-shell
test-unit-shell: ## Run shunit2 tests under Pants
	./pants filter --filter-target-type="shunit2_tests" :: \
	| xargs ./pants test

.PHONY: test-unit-js
test-unit-js: export COMPOSE_PROJECT_NAME := grapl-test-unit-js
test-unit-js: export COMPOSE_FILE := ./test/docker-compose.unit-tests-js.yml
test-unit-js: build-test-unit-js ## Build and run unit tests - JavaScript only
	test/docker-compose-with-error.sh
	$(MAKE) -C src/js/engagement_view test

.PHONY: test-typecheck
test-typecheck: ## Typecheck Python Code
	./pants typecheck ::

.PHONY: test-integration
test-integration: export COMPOSE_PROJECT_NAME := $(COMPOSE_PROJECT_INTEGRATION_TESTS)
test-integration: build-test-integration ## Build and run integration tests
	$(WITH_LOCAL_GRAPL_ENV)
	export SHOULD_DEPLOY_INTEGRATION_TESTS=True  # This gets read in by `docker-compose.yml`'s pulumi
	$(MAKE) test-with-env EXEC_TEST_COMMAND="nomad/local/run_parameterized_job.sh integration-tests 8"

.PHONY: test-grapl-template-generator
test-grapl-template-generator:  # Test that the Grapl Template Generator spits out something compilable.
	./src/python/grapl-template-generator/grapl_template_generator_tests/integration_test.sh

.PHONY: test-e2e
test-e2e: export COMPOSE_PROJECT_NAME := $(COMPOSE_PROJECT_E2E_TESTS)
test-e2e: build-test-e2e ## Build and run e2e tests
	$(WITH_LOCAL_GRAPL_ENV)
	export SHOULD_DEPLOY_E2E_TESTS=True  # This gets read in by `docker-compose.yml`'s pulumi
	$(MAKE) test-with-env EXEC_TEST_COMMAND="nomad/local/run_parameterized_job.sh e2e-tests 6"

# This target is not intended to be used directly from the command line, it's
# intended for tests in docker-compose files that need the Grapl environment.
.PHONY: test-with-env-docker
test-with-env-docker: # (Do not include help text - not to be used directly)
	$(MAKE) test-with-env EXEC_TEST_COMMAND=test/docker-compose-with-error.sh

# This target is not intended to be used directly from the command line.
# Think of it as a Context Manager that:
# - Before test-time, brings up a `make up-detach`
# - After test-time, tears it all down and dumps artifacts.
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
		$(MAKE) dump-artifacts
		$(MAKE) down
	}
	# Ensure we call stop even after test failure, and return exit code from
	# the test, not the stop command.
	trap stopGrapl EXIT
	$(WITH_LOCAL_GRAPL_ENV)
	# Bring up the Grapl environment and detach
	$(MAKE) up-detach
	# Run tests and check exit codes from each test container
	$${EXEC_TEST_COMMAND}

##@ Lint 🧹

.PHONY: lint-rust
lint-rust: ## Run Rust lint checks
	cd src/rust; bin/format --check; bin/lint

.PHONY: lint-python
lint-python: ## Run Python lint checks
	./pants filter --target-type=python_library,python_tests :: | xargs ./pants lint

.PHONY: lint-shell
lint-shell: ## Run Shell lint checks
	./pants filter --target-type=shell_library,shunit2_tests :: | xargs ./pants lint

.PHONY: lint-prettier
lint-prettier: build-formatter ## Run ts/js/yaml lint checks
	# `docker-compose run` will also propagate the correct exit code.
	# We could explore tossing `docker-compose` and switching to `docker run`,
	# like `grapl/grapl-rfcs`.
	docker-compose \
		--file docker-compose.formatter.yml \
		run --rm lint-prettier

.PHONY: lint-hcl
lint-hcl: ## Check to see if HCL files are formatted properly
	${NONROOT_DOCKER_COMPOSE_CHECK} hcl-lint

.PHONY: lint-proto
lint-proto: ## Lint all protobuf definitions
	${DOCKER_COMPOSE_CHECK} buf-lint

.PHONY: lint-proto-breaking
lint-proto-breaking: ## Check protobuf definitions for breaking changes
	${DOCKER_COMPOSE_CHECK} buf-breaking-change

.PHONY: lint
lint: lint-python lint-prettier lint-rust lint-shell lint-hcl lint-proto lint-proto-breaking ## Run all lint checks

##@ Formatting 💅

.PHONY: format-rust
format-rust: ## Reformat all Rust code
	cd src/rust; bin/format --update

.PHONY: format-python
format-python: ## Reformat all Python code
	./pants filter --target-type=python_library,python_tests :: | xargs ./pants fmt

.PHONY: format-shell
format-shell: ## Reformat all shell_libraries
	./pants filter --target-type=shell_library,shunit2_tests :: | xargs ./pants fmt

.PHONY: format-prettier
format-prettier: build-formatter ## Reformat js/ts/yaml
	# `docker-compose run` will also propagate the correct exit code.
	docker-compose \
		--file docker-compose.formatter.yml \
		run --rm format-prettier

.PHONY: format-hcl
format-hcl: ## Reformat all HCLs
	${NONROOT_DOCKER_COMPOSE_CHECK} hcl-format

.PHONY: format
format: format-python format-shell format-prettier format-rust format-hcl ## Reformat all code

##@ Local Grapl 💻

.PHONY: up
up: export COMPOSE_PROJECT_NAME="grapl"
up: build ## Build Grapl services and launch docker-compose up
	$(WITH_LOCAL_GRAPL_ENV)
	docker-compose -f docker-compose.yml up

.PHONY: up-detach
up-detach: build-local ## Bring up local Grapl and detach to return control to tty
	# Primarily used for bringing up an environment for integration testing.
	# For use with a project name consider setting COMPOSE_PROJECT_NAME env var
	# Usage: `make up-detach`
	$(WITH_LOCAL_GRAPL_ENV)
	# Start the Nomad agent
	$(MAKE) stop-nomad-detach; $(MAKE) start-nomad-detach
	# We use this target with COMPOSE_FILE being set pointing to other files.
	# Although it seems specifying the `--file` option overrides that, we'll
	# explicitly unset that here to avoid potential surprises.
	unset COMPOSE_FILE

	# TODO: This could potentially be replaced with a docker-compose run, but
	#  it doesn't have all these useful flags
	echo -e "\n--- Starting Pulumi"
	docker-compose \
		--file docker-compose.yml \
		up --force-recreate --always-recreate-deps --renew-anon-volumes \
		--exit-code-from pulumi \
		pulumi
	echo -e "\nPulumi complete"

.PHONY: down
down: ## docker-compose down - both stops and removes the containers
	$(WITH_LOCAL_GRAPL_ENV)
	# This is only for killing the lambda containers that Localstack
	# spins up in our network, but that docker-compose doesn't know
	# about. This must be the network that is used in Localstack's
	# LAMBDA_DOCKER_NETWORK environment variable.
	$(MAKE) stop-nomad-detach
	-docker kill $(shell docker ps --quiet --filter=network=grapl-network) || true
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

.PHONY: local-pulumi
local-pulumi: export COMPOSE_PROJECT_NAME="grapl"
local-pulumi:  ## launch pulumi via docker-compose up
	$(WITH_LOCAL_GRAPL_ENV)
	docker-compose -f docker-compose.yml run pulumi

.PHONY: start-nomad-detach
start-nomad-detach:  ## Start the Nomad environment, detached
	$(WITH_LOCAL_GRAPL_ENV)
	GRAPL_ROOT="${PWD}" nomad/local/start_detach.sh

.PHONY: stop-nomad-detach
stop-nomad-detach:  ## Stop Nomad CI environment
	nomad/local/stop_detach.sh

.PHONY: push
push: build-docker-images ## Push Grapl containers to supplied DOCKER_REGISTRY
	docker-compose --file=docker-compose.build.yml push

.PHONY: e2e-logs
e2e-logs: ## All docker-compose logs
	$(WITH_LOCAL_GRAPL_ENV)
	docker-compose $(EVERY_COMPOSE_FILE) --project-name $(COMPOSE_PROJECT_E2E_TESTS) logs -f

.PHONY: docker-kill-all
docker-kill-all:  # Kill all currently running Docker containers except registry
	# https://stackoverflow.com/a/46208493
	TO_KILL=$$(docker ps --all --quiet | grep -v -E $$(docker ps -aq --filter='name=grapl_local_registry' | paste -sd "|" -))
	docker kill $${TO_KILL}

.PHONY: populate-venv
populate-venv: ## Set up a Python virtualenv (you'll have to activate manually!)
	build-support/manage_virtualenv.sh populate

.PHONY: repl
repl: ## Run an interactive ipython repl that can import from grapl-common etc
	./pants --no-pantsd repl --shell=ipython src/python/repl

.PHONY: pulumi-prep
pulumi-prep: graplctl build-lambda-zips ux-tarball ## Prepare some artifacts in advance of running a Pulumi update (does not run Pulumi!)

.PHONY: update-buildkite-shared
update-buildkite-shared: ## Pull in changes from grapl-security/buildkite-common
	git subtree pull --prefix .buildkite/shared git@github.com:grapl-security/buildkite-common.git main --squash

.PHONY: build-docs
build-docs: ## Build the Sphinx docs
	./docs/build_docs.sh

.PHONY: local-graplctl-setup
local-graplctl-setup: ## Upload analyzers and data to a running Local Grapl (make sure to have done `make up` first)
	COMPOSE_PROJECT_NAME=grapl \
	COMPOSE_DOCKER_CLI_BUILD=1 \
	DOCKER_BUILDKIT=1 \
	docker-compose --env-file=local-grapl.env \
	--file=docker-compose.yml \
	--file=docker-compose.local-dev.yml \
	run local-graplctl
