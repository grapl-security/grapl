#
# Makefile for building Rust
#

ROOT_DIR = $(shell pwd)
RUST_BUILD ?= debug
ifeq ($(RUST_BUILD),release)
CARGO_BUILD_OPTS += --release
endif

.DEFAULT_GOAL := help

FMT_BLUE = \033[36m
FMT_PURPLE = \033[35m
FMT_BOLD = \033[1m
FMT_END = \033[0m

.PHONY: help
help: ## Print this help
	@printf '\n'
	@awk 'BEGIN {FS = ":.*##"; printf "Usage: make ${FMT_BLUE}<target>${FMT_END}\n"} \
		 /^[a-zA-Z0-9_-]+:.*?##/ { printf "  ${FMT_BLUE}%-46s${FMT_END} %s\n", $$1, $$2 } \
		 /^##@/ { printf "\n${FMT_BOLD}%s${FMT_END}\n", substr($$0, 5) } ' \
		 $(MAKEFILE_LIST)
	@printf '\n'

DOCKER_RUN := docker run \
	--rm \
	--interactive \
	--tty \
	--mount type=bind,source=${ROOT_DIR},target=/grapl/rust \
	--mount type=bind,source=${ROOT_DIR}/../proto,target=/grapl/proto \
	--mount type=volume,source=grapl-target,target=/grapl/target \
	--mount type=volume,source=grapl-cargo-cache,target=/grapl/cargo \
	--mount type=volume,source=grapl-rustup,target=/grapl/rustup \
	--user "$(shell id -u):$(shell id -g)" \
	--workdir /grapl/rust \
	grapl/rust-build-env


##@ Build 🔨

# Docker image that defines the environment to build Rust sources in
.PHONY: build-env-image
build-env-image:
	docker buildx build \
		--tag grapl/rust-build-env \
		- < build-env.Dockerfile

# Copy the build outputs to dist directory on the host.  We should switch to
# using the --out-dir features when that becomes stable:
# https://github.com/rust-lang/cargo/issues/6790. With that feature we could
# bind mount the dist/ directory to a mount point that --out-dir points to.
.PHONY: _copy-to-dist
_copy-to-dist:
	mkdir --parents --verbose "dist/$(RUST_BUILD)"
	# Clear dist directory and copy outputs from Docker volume
	rm -f "dist/$(RUST_BUILD)/*"
	$(DOCKER_RUN) find "/grapl/target/$(RUST_BUILD)" \
		-maxdepth 1 \
		-type f -executable \
		-exec cp {} /grapl/rust/dist/$(RUST_BUILD) \;
	# Copy web frontend to dist
	cp -r grapl-web-ui/frontend dist/$(RUST_BUILD)

.PHONY: build-cargo
build-cargo: build-env-image
	$(DOCKER_RUN) cargo build $(CARGO_BUILD_OPTS)
	$(MAKE) _copy-to-dist

.PHONY: build
build: build-cargo ## Build debug
	docker buildx bake

.PHONY: release
release: ## Build release
	$(MAKE) RUST_BUILD=release build

.PHONY: build-integration-test-image
build-integration-test-image: build-env-image ## Build container image for deploying integration tests
	$(DOCKER_RUN) /grapl/rust/build-int-tests.sh
	docker buildx bake int-tests

##@ Test 🧪

.PHONY: test
test: build-env-image ## Run 'cargo test'
	$(DOCKER_RUN) cargo test

.PHONY: test-integration
test-integration: build-env-image ## Run Rust integration tests
	$(DOCKER_RUN) cargo test \
		--features "node-identifier/integration,sqs-executor/integration,kafka-metrics-exporter/integration" \
		--test "*"

# Running Tarpaulin seems to touch Rust target files in such a way that the
# target cache is invalidated for all future rebuilds. Where running something
# like `cargo build` would be a very fast operation on subsequent builds,
# running `cargo tarpaulin` seems effectively taint that all that cache,
# causing full rebuilds.  For this reason, we define the environment to
# Tarpaulin a little differently: we don't mount the target directory. This way
# it won't ruin the build cache for other types of builds. Running `cargo
# tarpaulin` causes a fresh build each time, so we're not losing anything by
# not giving it its own target cache.
#
# Note: the default seccomp profile won't work for tarpaulin -
# https://github.com/xd009642/tarpaulin/issues/146
DOCKER_RUN_CODECOV := docker run \
	--rm \
	--interactive \
	--tty \
	--mount type=bind,source=${ROOT_DIR},target=/grapl/rust \
	--mount type=bind,source=${ROOT_DIR}/../proto,target=/grapl/proto \
	--mount type=volume,source=grapl-cargo-cache,target=/grapl/cargo \
	--mount type=volume,source=grapl-rustup,target=/grapl/rustup \
	--user "$(shell id -u):$(shell id -g)" \
	--workdir /grapl/rust \
	--security-opt seccomp=unconfined \
	grapl/rust-build-env

.PHONY: codecov
codecov: build-env-image ## Codecov
	mkdir --parents --verbose "dist/coverage"
	$(DOCKER_RUN_CODECOV) cargo install cargo-tarpaulin
	$(DOCKER_RUN_CODECOV) cargo tarpaulin --out=Xml --output-dir=/grapl/rust/dist/coverage

##@ Formatting 💅

.PHONY: format
format: ## Format
	bin/format --update

##@ Lint 🧹

.PHONY: lint
lint: ## Lint
	bin/format --check; bin/lint

##@ Utility ⚙

.PHONY: clean
clean: ## Remove dist directory and Docker volumes (target, cargo cache, etc.)
	rm -rf dist/
	docker volume rm grapl-target grapl-cargo-cache grapl-rustup
