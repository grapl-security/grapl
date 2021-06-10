#
# Makefile for developing using Docker
#

.DEFAULT_GOAL := help

TAG ?= latest
RUST_BUILD ?= debug
UID = $(shell id -u)
GID = $(shell id -g)
DOCKER_BUILDX_BAKE_OPTS ?=
COMPOSE_IGNORE_ORPHANS=1
COMPOSE_PROJECT_NAME ?=kafka-metrics-exporter
export

DOCKER_BUILDX_BAKE := docker buildx bake $(DOCKER_BUILDX_BAKE_OPTS)

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

FMT_BLUE = \033[36m
FMT_PURPLE = \033[35m
FMT_BOLD = \033[1m
FMT_END = \033[0m

.PHONY: help
help: ## Print this help
	@awk 'BEGIN {FS = ":.*##"; printf "Usage: make ${FMT_BLUE}<target>${FMT_END}\n"} \
		 /^[a-zA-Z0-9_-]+:.*?##/ { printf "  ${FMT_BLUE}%-46s${FMT_END} %s\n", $$1, $$2 } \
		 /^##@/ { printf "\n${FMT_BOLD}%s${FMT_END}\n", substr($$0, 5) } ' \
		 $(MAKEFILE_LIST)
	@printf '\n'


##@ Build 🔨
.PHONY: build-test-integration
build-test-integration: ## Build the integration tests
	$(DOCKER_BUILDX_BAKE) --file ./docker-compose.build.yml

##@ Test 👀
.PHONY: test-integration
test-integration: ## Execute integration tests
	$(MAKE) build-test-integration
	docker-compose -f docker-compose.integration-tests.yml up -d kafka zookeeper kafka-provision
	docker-compose -f docker-compose.integration-tests.yml up integration-tests
	docker-compose -f docker-compose.integration-tests.yml down
