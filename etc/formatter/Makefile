#
# Basically, just entrypoints to consume from the `grapl/format` image.
# Eventually, consider moving Rust and Pants formatting in here too.
#

.DEFAULT_GOAL := help

REPO_DIR=/mnt/grapl_repo_rw/

SHELL := bash
.ONESHELL:
# errexit nounset noclobber
.SHELLFLAGS := \
-e \
-u \
-o pipefail \
-c

##@ Lint 🧹

.PHONY: lint-js
lint-js: ## Run js lint checks
	cd $(REPO_DIR)/src/js; bin/format.sh --check

##@ Formatting 💅

.PHONY: format-js
format-js: ## Reformat all js/ts code
	cd $(REPO_DIR)/src/js; bin/format.sh --update