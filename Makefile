#
# Makefile for developing using Docker
#

TAG ?= latest
TARGET ?= debug
UID = $(shell id -u)
GID = $(shell id -g)

export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

#
# Just build
#

.PHONY: build
build: build-all ## alias for `build-aws`

.PHONY: build-all
build-all: ## build all targets (incl. local, test, zip)
	docker buildx bake \
		 -f docker-compose.Makefile.yml \
		 -f docker-compose.unit-tests.yml \
		 -f docker-compose.integration-tests.yml \
		 -f docker-compose.zips.yml

.PHONY: build-unit-tests
build-unit-tests:
	docker-compose -f docker-compose.unit-tests.yml build --parallel
	# docker buildx bake -f docker-compose.unit-tests.yml

.PHONY: build-integration-tests
build-integration-tests:
	docker buildx bake -f docker-compose.Makefile.yml -f docker-compose.integration-tests.yml

.PHONY: build-local
build-local: ## build services for local Grapl
	docker buildx bake -f docker-compose.Makefile.yml

.PHONY: build-aws
build-aws: ## build services for Grapl in AWS
	docker buildx bake -f docker-compose.zips.yml

#
# Test
#

.PHONY: test-unit
test-unit: build-unit-tests ## build and run unit tests
	test/docker-compose-with-error.sh -f docker-compose.unit-tests.yml

.PHONY: test-integration
test-integration: build-integration-tests ## build and run integration tests
	docker-compose -f docker-compose.Makefile.yml up --force-recreate -d
	# save exit code to allow for `make down` in event of test failure
	test/docker-compose-with-error.sh -f docker-compose.integration-tests.yml; \
	EXIT_CODE=$$?; \
	docker-compose -f docker-compose.Makefile.yml logs; \
	$(MAKE) down; \
	exit $$EXIT_CODE

.PHONY: lint
lint: ## Run lint checks
	cd src/rust; cargo fmt -- --check
	./etc/ci_scripts/py_lint.sh --check-only

#
# else
#

.PHONY: clean
clean: ## Prune all docker build cache
	docker builder prune -a -f

.PHONY: clean-sccache
clean-sccache: ## Prune docker mount cache (used for Rust caching)
	docker builder prune --filter type=exec.cachemount

.PHONY: release
release: ## make zip with cargo --release
	$(MAKE) TARGET=release zip

.PHONY: zip
zip: ## Generate zips for use in AWS
	docker buildx bake -f docker-compose.zips.yml
	UID=$(UID) GID=$(GID) TAG=$(TAG) \
		docker-compose -f docker-compose.zips.yml up

.PHONY: up
up: build-local ## build local services and docker-compose up
	docker-compose -f docker-compose.Makefile.yml up

.PHONY: down
down: ## docker-compose down --remove-orphans
	docker-compose -f docker-compose.Makefile.yml down --remove-orphans

.PHONY: help
help: ## print this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9_-]+:.*?## / {gsub("\\\\n",sprintf("\n%22c",""), $$2);printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
