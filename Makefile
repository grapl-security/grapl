#
# Makefile for developing using Docker
#

TAG ?= latest
# USER = $(shell id -u):$(shell id -g)
UID = $(shell id -u)
GID = $(shell id -g)
TARGET ?= debug
ifeq ($(TARGET),release)
	CARGO_ARGS += --release
endif

WORKDIR = /grapl
RUST_SRC_MOUNT ?= -v "$(CURDIR)/src/rust":$(WORKDIR)
DEV_RUST_IMAGE_NAME = grapl/rust-base:$(TAG)
RUST_SERVICES = analyzer-dispatcher \
				generic-subgraph-generator \
				graph-merger \
				metric-forwarder \
				node-identifier \
				node-identifier-retry-handler \
				sysmon-subgraph-generator \
				osquery-subgraph-generator
				
PYTHON_SERVICES = analyzer-executor \
				  dgraph-ttl \
				  engagement-creator \
				  engagement-edge \
				  model-plugin-deployer \
				  swarm-lifecycle-event-handler

export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

DOCKER_RUN := docker run --rm --user "$(UID):$(GID)" -w "$(WORKDIR)" -t

#
# Rust targets
#

.PHONY: build-rust-image
build-rust-image:
	cat ./src/rust/Dockerfile.Makefile | docker build --target base -t $(DEV_RUST_IMAGE_NAME) -

.PHONY: build-rust-image-sccache
build-rust-image-sccache:
	cat ./src/rust/Dockerfile.Makefile | docker build --target sccache -t $(DEV_RUST_IMAGE_NAME) -

.PHONY: build-rust
build-rust: build-rust-image ## Rust: cargo build (debug)
	$(DOCKER_RUN) $(RUST_SRC_MOUNT) $(DEV_RUST_IMAGE_NAME) cargo build $(CARGO_ARGS)

.PHONY: build-rust-sccache
build-rust-sccache: build-rust-image-sccache ## Rust: cargo build w/ sccache (debug)
	$(DOCKER_RUN) $(RUST_SRC_MOUNT) -v ${HOME}/.cache/sccache:/sccache $(DEV_RUST_IMAGE_NAME) bash -c 'cargo build $(CARGO_ARGS) && sccache -s'

.PHONY: test-unit-rust
test-unit-rust: build-rust-image ## Rust: cargo test
	$(DOCKER_RUN_RUST) $(RUST_SRC_MOUNT) $(DEV_RUST_IMAGE_NAME) cargo test

# For each service create a make target to crate the zip package for it
.PHONY: zip-rust $(addprefix zip-rust-,$(RUST_SERVICES))
zip-rust: $(addprefix zip-rust-,$(RUST_SERVICES))

$(addprefix zip-rust-,$(RUST_SERVICES)):
	$(eval $@_NAME := $(patsubst zip-rust-%,%,$@))
	$(eval $@_TMPDIR := $(shell mktemp -d))
	ln -s -f "$(CURDIR)/src/rust/target/x86_64-unknown-linux-musl/$(TARGET)/$($@_NAME)" "$($@_TMPDIR)/bootstrap"
	zip -q9 -dg "$($@_TMPDIR)/$($@_NAME)-$(TAG).zip" "$($@_TMPDIR)/bootstrap"
	mv "$($@_TMPDIR)/$($@_NAME)-$(TAG).zip" "$(CURDIR)/src/js/grapl-cdk/zips/$($@_NAME)-$(TAG).zip"
	rm -rf "$($@_TMPDIR)"

# .PHONY: clean-rust
# clean-rust: build-rust-image ## Rust: cargo clean
# 	$(DOCKER_RUN) $(RUST_SRC_MOUNT) $(DEV_RUST_IMAGE_NAME) cargo clean

# Targets for creating the deploy images for local Grapl
# .PHONY: build-deploy-image-rust $(addprefix build-deploy-image-rust-,$(RUST_SERVICES))
# build-deploy-image-rust: $(addprefix build-deploy-image-rust-,$(RUST_SERVICES))

# $(addprefix build-deploy-image-rust-,$(RUST_SERVICES)):
# 	$(eval $@_NAME := $(patsubst build-deploy-image-rust-%,%,$@))
# 	docker build \
# 		--build-arg TARGET=$(TARGET) \
# 		--target $($@_NAME)-deploy \
# 		-t grapl/grapl-$($@_NAME):$(TAG) \
# 		-f $(CURDIR)/src/rust/Dockerfile.Makefile \
# 		"$(CURDIR)/src/rust/target/x86_64-unknown-linux-musl"; \



# Create a make target for each service name that builds and creates zip and Docker deployment image
# .PHONY: $(RUST_SERVICES)
# $(RUST_SERVICES): build-rust
# 	$(MAKE) build-deploy-image-rust-$@



#
# Python
#

# DOCKER_BUILD_PYTHON := \
# 	docker build \
# 		-f $(CURDIR)/src/python/Dockerfile.Makefile

# .PHONY: build-python-grapl-analyzerlib
# build-python-analyzerlib:
# 	$(DOCKER_BUILD_PYTHON) \
# 		--target grapl-analyzerlib-build \
# 		-t grapl/grapl-analyzerlib:$(TAG) \
# 		$(CURDIR)/src

# .PHONY: build-test-python-grapl-analyzerlib
# build-test-python-grapl-analyzerlib:
# 	$(DOCKER_BUILD_PYTHON) \
# 		--target grapl-analyzerlib-test \
# 		-t grapl/grapl-analyzerlib-test:$(TAG) \
# 		$(CURDIR)/src

# .PHONY: build-python $(addprefix build-python-,$(PYTHON_SERVICES))
# build-python: $(addprefix build-python-,$(PYTHON_SERVICES))

# $(addprefix build-python-,$(PYTHON_SERVICES)): build-python-grapl-analyzerlib
# 	$(eval $@_NAME := $(patsubst build-python-%,%,$@))
# 	$(DOCKER_BUILD_PYTHON) \
# 		--target $($@_NAME)-build \
# 		-t grapl/grapl-$($@_NAME)-build:$(TAG) \
# 		$(CURDIR)/src

# .PHONY: build-test-python-all build-test-python-graph-descriptions build-test-python-grapl-common
# .PHONY: $(addprefix build-test-python-,$(PYTHON_SERVICES))
# build-test-python-all: build-test-python-graph-descriptions
# build-test-python-all: build-test-python-grapl-common
# build-test-python-all: build-test-python-grapl-analyzerlib
# build-test-python-all: $(addprefix build-test-python-,$(PYTHON_SERVICES))

# $(addprefix build-test-python-,$(PYTHON_SERVICES)) \
# build-test-python-graph-descriptions \
# build-test-python-grapl-common: build-python-grapl-analyzerlib
# 	$(eval $@_NAME := $(patsubst build-test-python-%,%,$@))
# 	$(DOCKER_BUILD_PYTHON) \
# 		--target $($@_NAME)-test \
# 		-t grapl/grapl-$($@_NAME)-test:$(TAG) \
# 		$(CURDIR)/src

# .PHONY: build-test-python-all
# build-test-python-all:
# 	docker-compose -f docker-compose.unit-tests.yml build

.PHONY: zip-python $(addprefix zip-python-,$(PYTHON_SERVICES))
zip-python: $(addprefix zip-python-,$(PYTHON_SERVICES))

$(addprefix zip-python-,$(PYTHON_SERVICES)):
	$(eval $@_NAME := $(patsubst zip-python-%,%,$@))
	$(eval $@_TMPDIR := $(shell mktemp -d))
	$(DOCKER_RUN) \
		-v "$($@_TMPDIR)":$(WORKDIR) \
		grapl/grapl-$($@_NAME)-build:$(TAG)
	mv "$($@_TMPDIR)/lambda.zip" "$(CURDIR)/src/js/grapl-cdk/zips/$($@_NAME)-$(TAG).zip"
	rm -rf "$($@_TMPDIR)"

# .PHONY: deploy-img-python-all $(addprefix deploy-img-python-,$(PYTHON_SERVICES))
# deploy-img-python-all: $(addprefix deploy-img-python-,$(PYTHON_SERVICES))

# $(addprefix deploy-img-python-,$(PYTHON_SERVICES)):
# 	$(eval $@_NAME := $(patsubst deploy-img-python-%,%,$@))
# 	$(DOCKER_BUILD_PYTHON) \
# 		--target $($@_NAME)-deploy \
# 		-t grapl/grapl-$($@_NAME):$(TAG) \
# 		$(CURDIR)/src

# .PHONY: $(PYTHON_SERVICES)
# $(PYTHON_SERVICES): 
# 	$(MAKE) build-python-$(@)
# 	$(MAKE) deploy-img-python-$(@)

# .PHONY: test-unit-python test-unit-python-graph-descriptions test-unit-python-grapl-common test-unit-python-grapl-analyzerlib
# .PHONY: $(addprefix test-unit-python-,$(PYTHON_SERVICES))
# test-unit-python: build-test-python-all
# test-unit-python: $(addprefix test-unit-python-,$(PYTHON_SERVICES))
# test-unit-python: test-unit-python-graph-descriptions
# test-unit-python: test-unit-python-grapl-common
# test-unit-python: test-unit-python-grapl-analyzerlib

# $(addprefix test-unit-python-,$(PYTHON_SERVICES)): build-python-grapl-analyzerlib
# 	$(eval $@_NAME := $(patsubst test-unit-python-%,%,$@))
# 	docker run --rm -t \
# 		-t grapl/grapl-$($@_NAME)-test:$(TAG)

#
# JS
#

# .PHONY: build-js-graphql-endpoint
# build-js-graphql-endpoint:
# 	$(eval $@_NAME := $(patsubst build-js-%,%,$@))
# 	docker build \
# 		-f $(CURDIR)/src/js/graphql_endpoint/Dockerfile.Makefile \
# 		--target $($@_NAME)-build \
# 		-t grapl/grapl-$($@_NAME)-build:$(TAG) \
# 		$(CURDIR)/src/js/graphql_endpoint

.PHONY: zip-js-graphql-endpoint
zip-js-graphql-endpoint:
	$(eval $@_NAME := $(patsubst zip-js-%,%,$@))
	$(eval $@_TMPDIR := $(shell mktemp -d))
	$(DOCKER_RUN) \
		--user "$(UID):$(GID)" \
		-v "$($@_TMPDIR)":$(WORKDIR) \
		grapl/grapl-$($@_NAME)-build:$(TAG)
	mv "$($@_TMPDIR)/lambda.zip" "$(CURDIR)/src/js/grapl-cdk/zips/$($@_NAME)-$(TAG).zip"
	rm -rf "$($@_TMPDIR)"

# .PHONY: deploy-img-js-graphql-endpoint
# deploy-img-js-graphql-endpoint:
# 	$(eval $@_NAME := $(patsubst deploy-img-js-%,%,$@))
# 	docker build \
# 		-f $(CURDIR)/src/js/graphql_endpoint/Dockerfile.Makefile \
# 		--target $($@_NAME)-deploy \
# 		-t grapl/grapl-$($@_NAME):$(TAG) \
# 		$(CURDIR)/src/js/graphql_endpoint

# .PHONY: graphql-endpoint
# graphql-endpoint:
# 	$(MAKE) build-js-$(@)
# 	$(MAKE) deploy-img-js-$(@)

#
# Tests
#

.PHONY: built-unit-tests
built-unit-tests:
	docker-compose -f docker-compose.unit-tests.yml build --parallel

.PHONY: built-integration-tests
built-integration-tests:
	docker-compose -f docker-compose.Makefile.yml -f docker-compose.integration-tests.yml build --parallel

#
# Targets for all
#

.PHONY: build
build: build-rust
	docker-compose -f docker-compose.Makefile.yml build --parallel

# .PHONY: build-aws
# build-aws: build-rust build-python build-js-graphql-endpoint

# .PHONY: release
# release:
# 	$(MAKE) TARGET=release build

.PHONY: clean
clean: ## Delete all "grapl/*" Docker images
	docker rmi --force $$(docker images "grapl/*" --format "{{.ID}}")

.PHONY: zip ## Create zips from pre-built outputs
zip: zip-rust zip-python zip-js-graphql-endpoint ## Create the zip for deployment to AWS (cdk/zips) - debug

.PHONY: aws
aws: build ## Build sources (debug) and create zips
	$(MAKE) zip

# .PHONY: aws-services
# aws-services: $(RUST_SERVICES) $(PYTHON_SERVICES) graphql-endpoint ## Build sources (debug), create zips and Docker deploy images

.PHONY: up
up: ## build local services and docker-compose up
	docker-compose -f docker-compose.Makefile.yml build --parallel
	docker-compose -f docker-compose.Makefile.yml up

.PHONY: down
down: ## docker-compose down --remove-orphans
	docker-compose -f docker-compose.Makefile.yml down --remove-orphans

.PHONY: test-unit
test-unit: built-unit-tests ## build and run unit tests
	docker-compose -f docker-compose.unit-tests.yml up
	# check for container exit codes other than 0
	for test in $$(docker-compose -f docker-compose.unit-tests.yml ps -q); do\
		docker inspect -f "{{ .State.ExitCode }}" $$test | grep -q ^0;\
		if [ $$? -ne 0 ]; then exit 1; fi;\
	done

.PHONY: test-integration
test-integration: built-integration-tests ## build and run integration tests
	docker-compose -f docker-compose.Makefile.yml up -d;\
	docker-compose -f docker-compose.integration-tests.yml up;\
	ERR=0;\
	for test in $$(docker-compose -f docker-compose.integration-tests.yml ps -q); do\
		docker inspect -f "{{ .State.ExitCode }}" $$test | grep -q ^0;\
		if [ $$? -ne 0 ]; then ERR=1; fi;\
	done;\
	$(MAKE) down
	exit $$ERR;

.PHONY: help
help: ## print this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9_-]+:.*?## / {gsub("\\\\n",sprintf("\n%22c",""), $$2);printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
