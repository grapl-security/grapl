#
# Makefile for developing using Docker
#

.DEFAULT_GOAL := help

# This variable is used in a few places, most notably
# docker-bake.hcl. You can read more about it there, but the TL;DR is
# that you'll need to set this to a proper version (not "", "latest",
# or "dev") in order to generate release builds of our services to use
# in a production deployment. Defaulting to `dev` is fine for
# day-to-day development and local testing, though (and in fact is
# required for our local usage of Nomad, because Nomad won't resolve a
# `latest` tag from the host machine.)
IMAGE_TAG ?= dev
RUST_BUILD ?= debug
UID = $(shell id --user)
GID = $(shell id --group)
PWD = $(shell pwd)
GRAPL_ROOT = ${PWD}
DIST_FOLDER = $(GRAPL_ROOT)/dist
COMPOSE_USER=${UID}:${GID}
COMPOSE_IGNORE_ORPHANS=1
COMPOSE_PROJECT_NAME ?= grapl
ETH0_ADDRESS := $(shell ip address show dev eth0 | grep "inet\b" | awk '{ print $$2 }' | awk -F/ '{ print $$1 }')

export

export EVERY_COMPOSE_FILE=--file docker-compose.yml \
	--file ./test/docker-compose.unit-tests-js.yml \

# This is used to send docker traces to Jaeger. This is primarily useful for debugging build time performance
ifdef WITH_TRACING
buildx_builder_args := --builder=grapl-tracing-builder
endif

# Helper macro to make using the HCL file for builds less
# verbose. Once we get rid of docker-compose.yml, we can just use
# `docker buildx bake`, since it will pick up the HCL file
# automatically.
DOCKER_BUILDX_BAKE_HCL := docker buildx bake --file=docker-bake.hcl $(buildx_builder_args)

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

# Our images are labeled; we can use this to help filter various
# Docker prune / rm / etc. commands to only touch our stuff.
#
# This is set in docker-bake.hcl
DOCKER_FILTER_LABEL := org.opencontainers.image.vendor="Grapl, Inc."

# Run a Pants goal across all Python files
PANTS_PYTHON_FILTER := ./pants filter --target-type=python_sources,python_tests :: | xargs ./pants
# Run a Pants goal across all shell files
PANTS_SHELL_FILTER := ./pants filter --target-type=shell_sources,shunit2_tests :: | xargs ./pants

# Helper macro for invoking a target from src/rust/Makefile
RUST_MAKE = $(MAKE) --directory=src/rust

# Helper macro for invoking a target from src/js/engagement_view/Makefile
ENGAGEMENT_VIEW_MAKE = $(MAKE) --directory=src/js/engagement_view

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
	@printf -- '  ${FMT_PURPLE}TARGETS${FMT_END}="typecheck-analyzer-executor typecheck-grapl-common" make typecheck\n'
	@printf -- '    to only run a subset of test targets.\n'
	@printf -- '\n'
	@printf -- '  ${FMT_PURPLE}KEEP_TEST_ENV=1${FMT_END} make test-integration\n'
	@printf -- '    to keep the test environment around after a test suite.\n'
	@printf -- '\n'
	@printf -- '  ${FMT_PURPLE}DEBUG_SERVICES${FMT_END}="graphql_endpoint grapl_e2e_tests" make test-e2e\n'
	@printf -- '    to launch the VSCode Debugger (see ${VSC_DEBUGGER_DOCS_LINK}).\n'
	@printf -- '\n'
	@printf -- '  ${FMT_PURPLE}WITH_TRACING=1${FMT_END} make build-local-infrastructure \n'
	@printf -- '    to send docker build traces to Jaeger (see docs/development/debugging.md).\n'
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

.PHONY: build-test-unit-js
build-test-unit-js:
	docker buildx bake \
		--file ./test/docker-compose.unit-tests-js.yml $(buildx_builder_args)

# Build Service Images and their Prerequisites
########################################################################
#
# Building of the various images we use for core Grapl SaaS services,
# our local-only images (e.g., pulumi, postgres), and any
# prerequisites they need (e.g., due to COPY directives in
# Dockerfiles) are defined here. The image builds are defined in
# docker-bake.hcl using groups. Similarly, the prerequisites that
# Pants knows how to build are defined using tags. The grapl-web-ui
# needs the compiled engagement-view assets in order for it to build.

.PHONY: build-service-pex-files
build-service-pex-files: ## Build all PEX files needed by Grapl SaaS services
	@echo "--- Building Grapl service PEX files"
	./pants --tag="service-pex" package ::

.PHONY: build-e2e-pex-files
build-e2e-pex-files:
# Any PEX tagged with `e2e-test-pex` is required for our image. This
# seems like the most straightforward way of capturing these
# dependencies at the moment.
	@echo "--- Building e2e PEX files"
	./pants --tag="e2e-test-pex" package ::

.PHONY: build-engagement-view
build-engagement-view: ## Build website assets to include in grapl-web-ui
	@echo "--- Building the engagement view"
	$(ENGAGEMENT_VIEW_MAKE) build-code
	cp -r \
		"${PWD}/src/js/engagement_view/build/." \
		"${PWD}/src/rust/grapl-web-ui/frontend/"

.PHONY: build-grapl-service-prerequisites

build-grapl-service-prerequisites: ## Build all assets needed for the creation of Grapl SaaS service container images
# The Python services need the PEX files
build-grapl-service-prerequisites: build-service-pex-files
# The grapl-web-ui service needs website assets
build-grapl-service-prerequisites: build-engagement-view

# This is used in our CI pipeline; see build_and_upload_images.sh
#
# Also see the `push-to-cloudsmith` group in docker-bake.hcl; any
# prerequisites of images in that group should be built by this
# target!
.PHONY: build-image-prerequisites
build-image-prerequisites: ## Build all dependencies that must be copied into our images that we push to our registry
build-image-prerequisites: build-grapl-service-prerequisites build-e2e-pex-files

.PHONY: build-local-infrastructure
build-local-infrastructure: build-grapl-service-prerequisites
	@echo "--- Building the Grapl SaaS service images and local-only images"
	$(DOCKER_BUILDX_BAKE_HCL) local-infrastructure

.PHONY: build-test-e2e
build-test-e2e: build-e2e-pex-files
	@echo "--- Building e2e testing image"
	$(DOCKER_BUILDX_BAKE_HCL) e2e-tests

.PHONY: build-test-integration
build-test-integration:
	@echo "--- Building integration test images"
	docker buildx bake integration-tests $(buildx_builder_args)

########################################################################

.PHONY: build-prettier-image
build-prettier-image:
	docker buildx bake --file ./docker-compose.check.yml prettier $(buildx_builder_args)

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

.PHONY: dump-artifacts-local
dump-artifacts-local:  # Run the script that dumps Nomad/Docker logs after test runs
	./pants run ./etc/ci_scripts/dump_artifacts -- \
		--compose-project="${COMPOSE_PROJECT_NAME}" \
		--dump-agent-logs

##@ Test 🧪

# Unit Tests
########################################################################

.PHONY: test-unit
test-unit: test-unit-js
test-unit: test-unit-python
test-unit: test-unit-rust
# NOTE: Intentionally *NOT* adding `test-unit-rust-coverage`; see below
test-unit: test-unit-shell
test-unit: ## Build and run all unit tests

.PHONY: test-unit-js
test-unit-js: test-unit-engagement-view
test-unit-js: test-unit-graphql-endpoint
test-unit-js: ## Build and run unit tests - JavaScript only

.PHONY: test-unit-graphql-endpoint
test-unit-graphql-endpoint: | dist
test-unit-graphql-endpoint: build-test-unit-js
test-unit-graphql-endpoint: export COMPOSE_PROJECT_NAME := grapl-test-unit-js
test-unit-graphql-endpoint: export COMPOSE_FILE := ./test/docker-compose.unit-tests-js.yml
test-unit-graphql-endpoint: ## Test Graphql Endpoint
	test/docker-compose-with-error.sh

.PHONY: test-unit-engagement-view
test-unit-engagement-view: ## Test Engagement View
	$(ENGAGEMENT_VIEW_MAKE) run-tests

.PHONY: test-unit-python
# Long term, it would be nice to organize the tests with Pants
# tags, rather than pytest tags
# If you need to `pdb` these tests, add a `--debug` after `./pants test`
test-unit-python: ## Run Python unit tests under Pants
	./pants filter --filter-target-type="python_tests" :: \
	| xargs ./pants --tag="-needs_work" test --pytest-args="-m \"not integration_test\""

.PHONY: test-unit-rust
test-unit-rust: ## Build and run unit tests - Rust only (not for CI)
# This does *NOT* gather coverage statistics; see
# test-unit-rust-coverage for that
	$(RUST_MAKE) test

.PHONY: test-unit-shell
test-unit-shell: ## Run shunit2 tests under Pants
	./pants filter --filter-target-type="shunit2_tests" :: \
	| xargs ./pants test

########################################################################

# NOTE: This is a separate target intended for use in CI only because
# gathering coverage statistics with Tarpaulin takes a long time
# (*everything* must be recompiled, even if nothing changed) making it
# less-than-ideal for day-to-day developer usage.
#
# As such, it is intentionally *NOT* specified as a prerequisite for
# the `test-unit` target.

# Unfortunately, we must ensure that the `dist` directory is present
# for this to work.
test-unit-rust-coverage: | dist
test-unit-rust-coverage: ## Run Rust unit tests and gather coverage statistics (CI only)
	$(RUST_MAKE) coverage

########################################################################

.PHONY: typecheck
typecheck: ## Typecheck Python Code
	./pants check ::

.PHONY: test-integration
test-integration: build-local-infrastructure
test-integration: build-test-integration
test-integration: export COMPOSE_PROJECT_NAME := $(COMPOSE_PROJECT_INTEGRATION_TESTS)
test-integration: ## Build and run integration tests
	@$(WITH_LOCAL_GRAPL_ENV)
	$(MAKE) test-with-env EXEC_TEST_COMMAND="nomad/bin/run_parameterized_job.sh integration-tests 9"

.PHONY: test-grapl-template-generator
test-grapl-template-generator:  # Test that the Grapl Template Generator spits out something compilable.
	./src/python/grapl-template-generator/grapl_template_generator_tests/integration_test.sh

.PHONY: test-e2e
test-e2e: build-local-infrastructure
test-e2e: build-test-e2e
test-e2e: export COMPOSE_PROJECT_NAME := $(COMPOSE_PROJECT_E2E_TESTS)
test-e2e: ## Build and run e2e tests
	@$(WITH_LOCAL_GRAPL_ENV)
	$(MAKE) test-with-env EXEC_TEST_COMMAND="nomad/bin/run_parameterized_job.sh e2e-tests 6"

# This target is not intended to be used directly from the command line.
# Think of it as a Context Manager that:
# - Before test-time, brings up a `make up`
# - After test-time, tears it all down and dumps artifacts.
.PHONY: test-with-env
test-with-env: # (Do not include help text - not to be used directly)
	stopGrapl() {
		# skip if KEEP_TEST_ENV is set
		if [[ -z "${KEEP_TEST_ENV}" ]]; then
			@echo "Tearing down test environment"
		else
			@echo "Keeping test environment" && return 0
		fi
		# Unset COMPOSE_FILE to help ensure it will be ignored with use of --file
		unset COMPOSE_FILE
		$(MAKE) dump-artifacts-local
		$(MAKE) down
	}
	# Ensure we call stop even after test failure, and return exit code from
	# the test, not the stop command.
	trap stopGrapl EXIT
	@$(WITH_LOCAL_GRAPL_ENV)
	# Bring up the Grapl environment and detach
	$(MAKE) _up
	# Run tests and check exit codes from each test container
	@echo "--- Executing test with environment"
	$${EXEC_TEST_COMMAND}

##@ Lint 🧹

.PHONY: lint
lint: lint-build
lint: lint-docker
lint: lint-hcl
lint: lint-prettier
lint: lint-proto
lint: lint-proto-breaking
lint: lint-python
lint: lint-rust
lint: lint-shell
lint: ## Run all lint checks

.PHONY: lint-build
lint-build: ## Lint Pants BUILD files
	./pants update-build-files --check

.PHONY: lint-docker
lint-docker: ## Lint Dockerfiles with Hadolint
	./pants filter --target-type=docker_image :: \
		| xargs ./pants lint

.PHONY: lint-hcl
lint-hcl: ## Check to see if HCL files are formatted properly
	${NONROOT_DOCKER_COMPOSE_CHECK} hcl-lint

.PHONY: lint-prettier
lint-prettier: build-prettier-image
lint-prettier: ## Run ts/js/yaml lint checks
	${NONROOT_DOCKER_COMPOSE_CHECK} prettier-lint

.PHONY: lint-proto
lint-proto: ## Lint all protobuf definitions
	${DOCKER_COMPOSE_CHECK} buf-lint

.PHONY: lint-proto-breaking
lint-proto-breaking: ## Check protobuf definitions for breaking changes
	${DOCKER_COMPOSE_CHECK} buf-breaking-change

.PHONY: lint-python
lint-python: ## Run Python lint checks
	$(PANTS_PYTHON_FILTER) lint

.PHONY: lint-rust
lint-rust: lint-rust-clippy
lint-rust: lint-rust-rustfmt
lint-rust: ## Run Rust lint checks

.PHONY: lint-rust-clippy
lint-rust-clippy: ## Run Clippy on Rust code
	$(RUST_MAKE) lint-clippy

.PHONY: lint-rust-rustfmt
lint-rust-rustfmt: ## Check to see if Rust code is properly formatted
	$(RUST_MAKE) lint-rustfmt

.PHONY: lint-shell
lint-shell: ## Run Shell lint checks
	$(PANTS_SHELL_FILTER) lint

##@ Formatting 💅

.PHONY: format
format: format-build
format: format-hcl
format: format-prettier
format: format-python
format: format-rust
format: format-shell
format: ## Reformat all code

.PHONY: format-build
format-build: ## Reformat all Pants BUILD files
	./pants update-build-files --no-update-build-files-fix-safe-deprecations

.PHONY: format-hcl
format-hcl: ## Reformat all HCL files
	${NONROOT_DOCKER_COMPOSE_CHECK} hcl-format

.PHONY: format-prettier
format-prettier: build-prettier-image
format-prettier: ## Reformat js/ts/yaml
	${NONROOT_DOCKER_COMPOSE_CHECK} prettier-format

.PHONY: format-python
format-python: ## Reformat all Python code
	$(PANTS_PYTHON_FILTER) fmt

.PHONY: format-rust
format-rust: ## Reformat all Rust code
	$(RUST_MAKE) format

.PHONY: format-shell
format-shell: ## Reformat all shell code
	$(PANTS_SHELL_FILTER) fmt

##@ Local Grapl 💻

.PHONY: up
up: ## Bring up local Grapl and detach to return control to tty
up: build-local-infrastructure _up

# NOTE: Internal target to decouple the building of images from the
# running of them. Do not invoke this directly.
.PHONY: _up
_up:
	# Primarily used for bringing up an environment for integration testing.
	# For use with a project name consider setting COMPOSE_PROJECT_NAME env var
	@echo "--- Deploying Nomad Infrastructure"
	@$(WITH_LOCAL_GRAPL_ENV)
	# Start the Nomad agent
	$(MAKE) stop-nomad-detach; $(MAKE) start-nomad-detach
	# We use this target with COMPOSE_FILE being set pointing to other files.
	# Although it seems specifying the `--file` option overrides that, we'll
	# explicitly unset that here to avoid potential surprises.
	unset COMPOSE_FILE

	# TODO: This could potentially be replaced with a docker-compose run, but
	#  it doesn't have all these useful flags
	@echo "--- Running Pulumi"
	docker-compose \
		--file docker-compose.yml \
		up --force-recreate --always-recreate-deps --renew-anon-volumes \
		--exit-code-from pulumi \
		pulumi

.PHONY: down
down: ## docker-compose down - both stops and removes the containers
	@$(WITH_LOCAL_GRAPL_ENV)
	# This is only for killing the lambda containers that Localstack
	# spins up in our network, but that docker-compose doesn't know
	# about. This must be the network that is used in Localstack's
	# LAMBDA_DOCKER_NETWORK environment variable.
	$(MAKE) stop-nomad-detach
	docker-compose $(EVERY_COMPOSE_FILE) down --timeout=0
	@docker-compose $(EVERY_COMPOSE_FILE) --project-name $(COMPOSE_PROJECT_INTEGRATION_TESTS) down --timeout=0
	@docker-compose $(EVERY_COMPOSE_FILE) --project-name $(COMPOSE_PROJECT_E2E_TESTS) down --timeout=0

.PHONY: stop
stop: ## docker-compose stop - stops (but preserves) the containers
	@$(WITH_LOCAL_GRAPL_ENV)
	docker-compose $(EVERY_COMPOSE_FILE) stop

# This is a convenience target for our frontend engineers, to make the dev loop
# slightly less arduous for grapl-web-ui/engagement-view development.
# It will *rebuild* the JS/Rust grapl-web-ui dependences, and then 
# restart a currently-running `make up` web ui allocation, which will then
# retrieve the latest, newly-rebuilt Docker container.
#
# Will only work as expected as long as tag is "dev".
.PHONY: restart-web-ui
restart-web-ui: build-engagement-view  ## Rebuild web-ui image, and restart web-ui task in Nomad
	$(DOCKER_BUILDX_BAKE_HCL) grapl-web-ui
	source ./nomad/lib/nomad_cli_tools.sh
	nomad alloc restart "$$(nomad_get_alloc_id grapl-core web-ui)"

##@ Venv Management
########################################################################
.PHONY: generate-constraints
generate-constraints: ## Generates a constraints file from the requirements.txt file
	build-support/manage_virtualenv.sh regenerate-constraints

.PHONY: populate-venv
populate-venv: ## Set up a Python virtualenv from constraints file (you'll have to activate manually!)
	build-support/manage_virtualenv.sh populate

##@ Utility ⚙

# Preliminaries

dist:
	mkdir dist

.PHONY: clean
clean: clean-dist
clean: clean-artifacts
clean: clean-engagement-view
clean: ## Clean all generated artifacts

.PHONY: clean-all
clean-all: clean
clean-all: clean-docker
clean-all: clean-all-rust
clean-all: ## Clean all generated artifacts AND Docker-related resources

.PHONY: clean-dist
clean-dist: ## Clean out the `dist` directory
	rm -Rf dist

.PHONY: clean-docker
clean-docker: clean-docker-cache
clean-docker: clean-docker-containers
clean-docker: clean-docker-images
clean-docker: ## Clean all Docker-related resources

.PHONY: clean-artifacts
clean-artifacts: ## Remove all dumped artifacts from test runs
# See dump_artifacts.py
	rm -Rf test_artifacts

.PHONY: clean-docker-cache
clean-docker-cache:
	docker builder prune --all --force

.PHONY: clean-docker-cache-mount
clean-docker-cache-mount: ## Prune only the Buildkit cache mounts
# The best documentation I can find for this is right now is
# https://github.com/docker/cli/issues/2325#issuecomment-733975408
	docker builder prune --filter type=exec.cachemount
# TODO: worth adding any additional types for pruning?

clean-docker-containers: ## Remove all running Grapl containers
	docker ps \
		--filter=label=$(DOCKER_FILTER_LABEL) \
	| xargs --no-run-if-empty docker rm --volumes --force

clean-docker-images: ## Remove all Grapl images
	docker images \
		--filter=label=$(DOCKER_FILTER_LABEL) \
		--quiet \
	| xargs --no-run-if-empty docker rmi --force

.PHONY: clean-engagement-view
clean-engagement-view:
	$(ENGAGEMENT_VIEW_MAKE) clean

clean-all-rust:
	$(RUST_MAKE) clean-all

########################################################################

.PHONY: local-pulumi
local-pulumi: export COMPOSE_PROJECT_NAME="grapl"
local-pulumi:  ## launch pulumi via docker-compose up
	@$(WITH_LOCAL_GRAPL_ENV)
	docker-compose -f docker-compose.yml run pulumi

.PHONY: start-nomad-detach
start-nomad-detach:  ## Start the Nomad environment, detached
	@$(WITH_LOCAL_GRAPL_ENV)
	nomad/local/start_detach.sh

.PHONY: stop-nomad-detach
stop-nomad-detach:  ## Stop Nomad CI environment
	nomad/local/stop_detach.sh

.PHONY: docker-kill-all
docker-kill-all:  # Kill all currently running Docker containers except registry
	# https://stackoverflow.com/a/46208493
	TO_KILL=$$(docker ps --all --quiet | grep -v -E $$(docker ps -aq --filter='name=grapl_local_registry' | paste -sd "|" -))
	docker kill $${TO_KILL}

.PHONY: repl
repl: ## Run an interactive ipython repl that can import from grapl-common etc
	./pants --no-pantsd repl --shell=ipython src/python/repl

.PHONY: build-docs
build-docs: ## Build the Sphinx docs
	./docs/build_docs.sh

.PHONY: generate-nomad-rust-client
generate-nomad-rust-client: ## Generate the Nomad rust client from OpenAPI
	./src/rust/bin/generate_nomad_rust_client.sh
	$(MAKE) format-rust
# TODO: If we ever break out a targeted `format-markdown` target, we
# should use that here.
	$(MAKE) format-prettier

.PHONY: setup-docker-tracing
buildx-tracing: ## This is a one-time setup for enabling docker buildx traces
	docker buildx create \
      --name grapl-tracing-builder \
      --driver docker-container \
      --driver-opt network=host \
      --driver-opt env.JAEGER_TRACE=localhost:6831 \
      --use

.PHONY: generate-sqlx-data
generate-sqlx-data:  # Regenerate sqlx-data.json based on queries made in Rust code
	./src/rust/bin/generate_sqlx_data.sh

dist/firecracker_kernel.tar.gz: firecracker/generate_firecracker_kernel.sh | dist
	./firecracker/generate_firecracker_kernel.sh

# TODO: Would be nice to be able to specify the input file prerequisites of
# this target and make non-PHONY. It's currently PHONY because otherwise,
# rebuilds would only occur if the dist/plugin-bootstrap-init dir were deleted.
# NOTE: While this target is PHONY, it *does* represent a real directory in 
# dist/
.PHONY: dist/plugin-bootstrap-init
dist/plugin-bootstrap-init: | dist  ## Build the Plugin Bootstrap Init (+ associated files) and copy it to dist/
	$(DOCKER_BUILDX_BAKE_HCL) plugin-bootstrap-init

# TODO: Would be nice to be able to specify the input file prerequisites of
# this target, once `dist/plugin-bootstrap-init` is non-PHONY
dist/firecracker_rootfs.tar.gz: dist/plugin-bootstrap-init | dist
	packer init -upgrade firecracker/packer/build-rootfs.pkr.hcl
	packer build \
	 	-var dist_folder="${GRAPL_ROOT}/dist" \
		firecracker/packer/build-rootfs.pkr.hcl