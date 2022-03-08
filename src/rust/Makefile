# Pull the version from our rust-toolchain.toml file; automatic
# synchronization FTW!
# TODO: Consider pulling this logic into a script for other uses
rust_version := $(shell grep channel rust-toolchain.toml | sed -E 's/channel = "(.*)"/\1/g')

.ONESHELL:
SHELL := bash
.SHELLFLAGS := -o errexit -o nounset -o pipefail -c

build_image_tag := grapl-rust-build-env:latest
docker_run := REPO_ROOT="$(abspath $(CURDIR)/../..)" IMAGE=$(build_image_tag) ./bin/run_in_container.sh

.PHONY: clean-all
clean-all: clean-image
clean-all: clean-volumes

########################################################################

.PHONY: clean-image
clean-image: ## Remove the build environment image
	docker rmi --force "${build_image_tag}" 2> /dev/null
	rm -f image

########################################################################
# Clean all volumes

# TODO: keep these names in sync with what the `docker run` invocation uses.
volumes := grapl-cargo-cache grapl-rustup grapl-target

.PHONY: clean-volumes
clean-volumes: ## Remove all volumes used by our build environment container

define make-clean-volume-rule
clean-volumes: clean-volume-$1

.PHONY: clean-volume-$1
clean-volume-$1: ## Remove the $1 volume
	docker volume remove --force $1
endef

$(foreach volume,${volumes},$(eval $(call make-clean-volume-rule,$(volume))))

########################################################################

format: ## Format all code
	bin/format --update

########################################################################

lint-rustfmt: ## Check formatting
	bin/format --check

lint-clippy: ## Run Clippy
	bin/lint

########################################################################

image: build-env.Dockerfile
image: rust-toolchain.toml
image: ## Build the build environment image
	docker buildx build \
		--build-arg=RUST_VERSION="${rust_version}" \
		--file=build-env.Dockerfile \
		--tag="${build_image_tag}" \
		.
		touch image

.PHONY: build
build: image
build: ## Build Rust code
	$(docker_run) cargo build

.PHONY: test
test: image
test: ## Run `cargo test`, without generating test coverage statistics
	$(docker_run) cargo test

.PHONY: coverage
coverage: image
coverage: ## Run `cargo tarpaulin` to generate test coverage statistics.
# NOTE: This invocation requires that `<REPO_ROOT>/dist` be mounted
# into the container. This is handled in `run_in_container.sh`; just
# be aware of this little wrinkle.
	$(docker_run) cargo install cargo-tarpaulin
	$(docker_run) cargo tarpaulin \
		--out=Xml \
		--output-dir=/dist/coverage
