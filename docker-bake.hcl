# TODO: Add the lint checks for the following conditions:
# - All container images really are accounted for in the "all" group
# - No group includes any target that starts with "_"
# - Targets that start with "_" can only appear in `inherits` lists
#   - Should *only* targets that begin with "_" appear in `inherits` lists?
# - All the targets in the "cloudsmith-images" group are configured
#   with the `upstream_aware_tag` function
# - All the targets *not* in the "cloudsmith-images" group are *not*
#   configured with the `upstream_aware_tag` function
# - Introspect all targets to find all context directories and ensure
#   that a .dockerignore file exists in each location

# Variables
########################################################################
# Variables in this section are intended to be set by users to
# influence the build.

# IMAGE_TAG is the "master variable"; if it is set to something other
# than "latest", we will build images suitable for production
# usage. If it is unset, "dev", or "latest", we'll be creating images for
# local usage only.
#
# In general, you'll only set this (via an environment variable when
# invoking `docker buildx bake`) when you want to set a real version
# tag and create images to push to Cloudsmith.
#
# For everyday local developer usage, you shouldn't need to change
# this value.
#
# See the `upstream_aware_tag` function below for additional
# information.
variable "IMAGE_TAG" {
  default = ""
}

# TODO: Document pushing to a sandbox repository, or rework the
# variables and functions to accommodate that.
# IMAGE_TAG=XXX RUST_BUILD=debug CONTAINER_REGISTRY=XXX/my-repo

# Variables below are generally not intended to be set by users. You
# can if you know what you're doing, but they're really meant to
# encapsulate some core logic for how we build. Override them at your
# own risk!
# ----------------------------------------------------------------------

# If IMAGE_TAG is either unset or "latest", or "dev", we are *not* doing builds
# for production usage. Production usage implies three things:
#
# - We're pushing our images to our Cloudsmith repository
# - We *never* going to use a "latest" tag. Ever. Only
#   explicitly-versioned images
# - Rust builds will use a "release" build profile
#
# NOTE: unset and "latest" are equivalent; "dev" is our local
# convention. We use Nomad and Nomad will not use a "latest"-tagged
# image from the local machine, so we have to have something that
# plays well for local developers.
#
variable "RELEASE_BUILD" {
  default = not(contains(["", "latest", "dev"], "${IMAGE_TAG}"))
}

# If this is a release build, we want to use the release profile for
# our Rust projects. Otherwise, we'll stick with the standard debug
# profile.
variable "RUST_BUILD" {
  default = RELEASE_BUILD ? "release" : "dev-local-grapl"
}

# This will be incorporated into the base image identifier for our
# Rust images. In general, it should correspond to the version in
# `src/rust/rust-toolchain.toml`, which we'll extract in our Makefile
# and pass in here. If something weird happens in the future where we
# need to override that for some reason, we can.
variable "RUST_VERSION" {
}

# This will be incorporated into the base image identifier for our
# Python images. In general, it should correspond to the version in
# `.python-version`, which we'll extract in our Makefile
# and pass in here. If something weird happens in the future where we
# need to override that for some reason, we can.
variable "PYTHON_VERSION" {
}

# This is the directory that certain artifacts will be deposited into
variable "DIST_DIR" {
}

# When performing a release build, we will tag our images with our
# "raw" Cloudsmith repository Docker registry address. We have a
# series of repositories that we promote containers through as they
# progress through our release pipeline; the "raw" one is the first
# stage, where all artifacts are initially pushed to.
variable "CONTAINER_REGISTRY" {
  default = "docker.cloudsmith.io/grapl/raw"
}

# Functions
########################################################################

# Note that our local testing setup assumes that containers are just
# named with their bare service name.
#
# (This also happens to be the convention for Docker Hub "library"
# images, which means that we'll never be able to accidentally push
# any of these images, because we don't have permission to push to any
# Docker Hub library repositories! We *do* own the "grapl" namespace
# in Docker Hub, though, which is why we don't name these images
# "grapl/${image_name}"; we *could* accidentally push those, which we
# don't want.)
function "upstream_aware_tag" {
  params = [image_name]
  result = RELEASE_BUILD ? "${CONTAINER_REGISTRY}/${image_name}:${IMAGE_TAG}" : local_only_tag("${image_name}")
}

# Images that are only intended for local usage should be tagged using
# this function.
#
# You can't push images to a remote registry if it doesn't have that
# registry as part of its tags, after all.
function "local_only_tag" {
  params = [image_name]
  result = "${image_name}:${IMAGE_TAG}"
}

# Groups
########################################################################

# Build everything by default.
#
# In general, you'll probably never really need this, but if you
# invoke a build without specifying a target, you'll definitely get
# what you want.
group "default" {
  targets = [
    "all"
  ]
}

# The services that will ultimately be deployed for Grapl in AWS
group "grapl-services" {
  # NOTE: Please keep this list sorted in alphabetical order
  targets = [
    "javascript-services",
    "python-services",
    "rust-services",
  ]
}

# This is the subset of images that we will build in CI and push to
# our Cloudsmith repository for provisioned infrastructure testing and
# deployment.
group "cloudsmith-images" {
  # NOTE: Please keep this list sorted in alphabetical order
  targets = [
    "grapl-services",
    "rust-integration-tests"
  ]
}

group "rust-services" {
  # NOTE: Please keep this list sorted in alphabetical order
  targets = [
    "event-source",
    "generator-dispatcher",
    "generator-execution-sidecar",
    "graph-merger",
    "grapl-web-ui",
    "kafka-retry",
    "node-identifier",
    "organization-management",
    "pipeline-ingress",
    "plugin-bootstrap",
    "plugin-registry",
    "plugin-work-queue",
    "graph-schema-manager",
    "uid-allocator",
  ]
}

group "python-services" {
  # NOTE: Please keep this list sorted in alphabetical order
  targets = [
    "provisioner"
  ]
}

group "javascript-services" {
  targets = [
    "graphql-endpoint"
  ]
}

# These are utility services that are used only for local (not AWS)
# deployments. As such, they should never be pushed to any remote
# image registries.
group "local-only-services" {
  # NOTE: Please keep this list sorted in alphabetical order
  targets = [
    "postgres",
    "pulumi",
    "scylladb"
  ]
}

# These are the images needed for running Grapl in a "local Grapl"
# context. Tests are intentionally excluded
group "local-infrastructure" {
  # NOTE: Please keep this list sorted in alphabetical order
  targets = [
    "grapl-services",
    "local-only-services"
  ]
}

group "python-integration-tests" {
  # NOTE: Please keep this list sorted in alphabetical order
  targets = [
    "python-integration-tests",
  ]
}

group "all-tests" {
  # NOTE: Please keep this list sorted in alphabetical order
  targets = [
    "python-integration-tests",
    "rust-integration-tests"
  ]
}

group "all" {
  # NOTE: Please keep this list sorted in alphabetical order
  targets = [
    "all-tests",
    "local-only-services",
    "grapl-services",
    "export-rust-build-artifacts-to-dist",
  ]
}

# Targets
########################################################################
#
# All targets whose name begins with an underscore ("_") are meant to
# be "base targets"; they are not meant to define an image, but rather
# a set of configuration values that can be inherited by other
# targets.
#
# Such targets should only appear in `inherits` arrays, and never in
# the `targets` list of any group.

# All our container images should ultimately inherit from this target,
# either directly or indirectly through another target.
target "_grapl-base" {
  # Define a set of standard OCI labels to attach to all images.
  #
  # See https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys
  labels = {
    "org.opencontainers.image.authors" = "https://graplsecurity.com"
    "org.opencontainers.image.source"  = "https://github.com/grapl-security/grapl",
    # In particular, this `vendor` label is used by various filters in
    # our top-level Makefile; if you change this, make sure to update
    # things over there, too.
    "org.opencontainers.image.vendor" = "Grapl, Inc."
  }
}

# Rust Services
# ----------------------------------------------------------------------

# All Rust services defined in src/rust/Dockerfile should inherit from
# this target.
target "_rust-base" {
  inherits = ["_grapl-base"]
  context  = "src"

  # Additional named contexts:
  # https://www.docker.com/blog/dockerfiles-now-support-multiple-build-contexts/
  contexts = {
    dist-ctx = "dist"
    etc-ctx  = "etc"
    test-ctx = "test"
  }
  dockerfile = "rust/Dockerfile"
  args = {
    RUST_BUILD   = "${RUST_BUILD}"
    RUST_VERSION = "${RUST_VERSION}"
  }
}

target "generator-dispatcher" {
  inherits = ["_rust-base"]
  target   = "generator-dispatcher-deploy"
  tags = [
    upstream_aware_tag("generator-dispatcher")
  ]
}

target "graph-merger" {
  inherits = ["_rust-base"]
  target   = "graph-merger-deploy"
  tags = [
    upstream_aware_tag("graph-merger")
  ]
}

target "grapl-web-ui" {
  inherits = ["_rust-base"]
  target   = "grapl-web-ui-deploy"
  tags = [
    upstream_aware_tag("grapl-web-ui")
  ]
}

target "event-source" {
  inherits = ["_rust-base"]
  target   = "event-source-deploy"
  tags = [
    upstream_aware_tag("event-source")
  ]
}

target "generator-execution-sidecar" {
  inherits = ["_rust-base"]
  target   = "generator-execution-sidecar-deploy"
  tags = [
    upstream_aware_tag("generator-execution-sidecar")
  ]
}

target "kafka-retry" {
  inherits = ["_rust-base"]
  target   = "kafka-retry-deploy"
  tags = [
    upstream_aware_tag("kafka-retry")
  ]
}

target "node-identifier" {
  inherits = ["_rust-base"]
  target   = "node-identifier-deploy"
  tags = [
    upstream_aware_tag("node-identifier")
  ]
}

target "organization-management" {
  inherits = ["_rust-base"]
  target   = "organization-management-deploy"
  tags = [
    upstream_aware_tag("organization-management")
  ]
}

target "pipeline-ingress" {
  inherits = ["_rust-base"]
  target   = "pipeline-ingress-deploy"
  tags = [
    upstream_aware_tag("pipeline-ingress")
  ]
}

target "plugin-bootstrap" {
  inherits = ["_rust-base"]
  target   = "plugin-bootstrap-deploy"
  tags = [
    upstream_aware_tag("plugin-bootstrap")
  ]
}

# A somewhat special target among the Rust targets, as it
# has an `output =` that dumps its contents into `dist/`.
target "export-rust-build-artifacts-to-dist" {
  inherits = ["_rust-base"]
  target   = "export-rust-build-artifacts-to-dist"
  output = [
    "type=local,dest=${DIST_DIR}"
  ]
}

target "plugin-registry" {
  inherits = ["_rust-base"]
  target   = "plugin-registry-deploy"
  tags = [
    upstream_aware_tag("plugin-registry")
  ]
}

target "plugin-work-queue" {
  inherits = ["_rust-base"]
  target   = "plugin-work-queue-deploy"
  tags = [
    upstream_aware_tag("plugin-work-queue")
  ]
}

target "graph-schema-manager" {
  inherits = ["_rust-base"]
  target   = "graph-schema-manager-deploy"
  tags = [
    upstream_aware_tag("graph-schema-manager")
  ]
}

target "uid-allocator" {
  inherits = ["_rust-base"]
  target   = "uid-allocator-deploy"
  tags = [
    upstream_aware_tag("uid-allocator")
  ]
}


# Python Services
# ----------------------------------------------------------------------

# All Python services defined in src/python/Dockerfile should inherit
# from this target.
target "_python-base" {
  inherits = ["_grapl-base"]
  contexts = {
    dist-ctx = "dist"
    etc-ctx  = "etc"
  }
  dockerfile = "src/python/Dockerfile"
  args = {
    PYTHON_VERSION = "${PYTHON_VERSION}"
  }
}

target "provisioner" {
  inherits = ["_python-base"]
  target   = "provisioner-deploy"
  tags = [
    upstream_aware_tag("provisioner")
  ]
}

# JavaScript Services
# ----------------------------------------------------------------------

target "graphql-endpoint" {
  inherits   = ["_grapl-base"]
  context    = "src/js/graphql_endpoint"
  dockerfile = "Dockerfile"
  target     = "graphql-endpoint-deploy"
  tags = [
    upstream_aware_tag("graphql-endpoint")
  ]
}

# Testing Images
# ----------------------------------------------------------------------

target "python-integration-tests" {
  inherits = ["_python-base"]
  target   = "integration-tests"
  tags = [
    local_only_tag("python-integration-tests")
  ]
}

target "rust-integration-tests" {
  inherits = ["_rust-base"]
  target   = "integration-tests"
  tags = [
    # Yes, we push this up to Cloudsmith to run tests against AWS
    # infrastructure; that's why we use `upstream_aware_tag`.
    upstream_aware_tag("rust-integration-tests")
  ]
}

# Local Testing Only
# ----------------------------------------------------------------------
# None of these are ever pushed to Cloudsmith.

target "pulumi" {
  inherits   = ["_grapl-base"]
  context    = "."
  dockerfile = "Dockerfile.pulumi"
  tags = [
    local_only_tag("local-pulumi")
  ]
}

target "postgres" {
  inherits   = ["_grapl-base"]
  context    = "postgres"
  dockerfile = "Dockerfile"
  tags = [
    local_only_tag("postgres-ext")
  ]
}

target "scylladb" {
  inherits   = ["_grapl-base"]
  context    = "scylladb"
  dockerfile = "Dockerfile"
  tags = [
    local_only_tag("scylladb-ext")
  ]
}
