# Building Grapl

This document describes Grapl's build system. Here you will find
instructions for building the Grapl source tree, running tests, and
running Grapl locally. This document also describes how the build
tools are used in our Github Actions based Continuous Integration (CI)
system.

## Building the source

Grapl uses Docker for build and test environments. All Grapl source
builds happen in Docker image builds. This has the added benefit of
enabling Grapl developers to easily spin up the entire Grapl stack
locally for a nice interactive development experience.

### Requirements

- [Docker Engine](https://docs.docker.com/engine/install/) (version 20.10 or later)
- [docker-compose](https://docs.docker.com/compose/install/)
- [GNU Make](https://www.gnu.org/software/make/)

### Getting started

Our Makefile defines a number of targets for building, testing and running
Grapl locally. A listing of helpful targets can be printed with `make help`:

```
$ make help
build                Alias for `services` (default)
build-all            Build all targets (incl. services, tests, zip)
build-services       Build Grapl services
build-aws            Build services for Grapl in AWS (subset of all services)
test-unit            Build and run unit tests
test-unit-rust       Build and run unit tests - Rust only
test-unit-python     Build and run unit tests - Python only
test-unit-js         Build and run unit tests - JavaScript only
test-typecheck       Build and run typecheck tests
test-integration     Build and run integration tests
test-e2e             Build and run e2e tests
test                 Run all tests
lint-rust            Run Rust lint checks
lint-python          Run Python lint checks
lint                 Run all lint checks
clean                Prune all docker build cache and remove Grapl containers and images
clean-mount-cache    Prune all docker mount cache (used by sccache)
release              'make build-services' with cargo --release
zip                  Generate zips for deploying to AWS (src/js/grapl-cdk/zips/)
deploy               CDK deploy to AWS
up                   Build Grapl services and launch docker-compose up
down                 docker-compose down
help                 Print this help
```

Examples:

- To kick off a local build (but no tests), run the following command:

```
make build
```

- To run all the unit tests, run the following command:

```
make test-unit
```

To run build and launch Grapl locally, run the following command

```
make up
```

### Environment variables

For convenience, the Makefile imports environment variables from a `.env` file.

The following environment variables can affect the build and test environments:

- `TAG` (default: `latest`) - This is the tag we'll use for all the Docker
  images. For local builds `latest` is fine. Production builds should have a
specific version e.g. `v1.2.3`. Users may want to use a tag that includes
version and/or branch information for tracking purposes (ex:
`v1.2.3-my_feature`). This value corresponds to the `graplVersion` parameter in
the CDK project for deploying to AWS, and is used to name the zip files in the
Make `zip` target.
- `CARGO_PROFILE` (default: `debug`) - Can either be `debug` or `release`. These
  roughly translate to the [Cargo
profiles](https://doc.rust-lang.org/cargo/reference/profiles.html) to be used
for Rust builds.
- `GRAPL_RUST_ENV_FILE` - File path to a shell script in to be sources for the
  Rust builds. This can be used to set and override environment variables,
which can be useful for things like settings for
[sccache](https://github.com/mozilla/sccache), which is used to for caching. It
is passed as a [Docker build
secret](https://docs.docker.com/develop/develop-images/build_enhancements/#new-docker-build-secret-information)
so it should be suitable secrets like S3 credentials for use with sccache.
- `DOCKER_BUILDX_BAKE_OPTS` - Docker images are built using [docker
  buildx](https://github.com/docker/buildx). You can pass additional arguments
to the `docker buildx build` commands by setting this option (ex: `--progress
plain`).

#### CDK deployment parameters

Arguments to the CDK deployment parameters can be supplied via environment
variables documented in [docs/setup/aws.md](./docs/setup/aws.md#configure). By
using `make deploy` to execute a CDK deploy, the environment variables can be
read from a `.env` in the root of the Grapl respository.

### sccache

By default, our builds will use Mozilla's
[sccache](https://github.com/mozilla/sccache) to cache builds in a [cache mount
type](https://github.com/moby/buildkit/blob/master/frontend/dockerfile/docs/syntax.md#run---mounttypecache).
This improves performance for local development experience as Rust sources
change.

Environment variables used by `sccache` can be supplied via the
`GRAPL_RUST_ENV_FILE` environment variable when running Make.

Examples:

- To disable `sccache` you can do the following:

```
$ echo "unset RUSTC_WRAPPER" > .rust_env.sh
$ export GRAPL_RUST_ENV_FILE=.rust_env.sh
$ make build
```

- To configure `sccache` to use S3 on a local minio server running on 172.17.0.100:8000:

```
$ cat <<EOF > .rust_env.sh
export SCCACHE_BUCKET=sccache
export AWS_ACCESS_KEY_ID=AKIAEXAMPLE
export AWS_SECRET_ACCESS_KEY="d2hhdCBkaWQgeW91IGV4cGVjdCB0byBmaW5kPwo="
export SCCACHE_DIR=/root/sccache
export SCCACHE_ENDPOINT="172.17.0.100:8000"
EOF
$ export GRAPL_RUST_ENV_FILE=.rust_env.sh
$ make build
```

## How it works

### Overview

[Docker Compose files](https://docs.docker.com/compose/compose-file/) are used to define:

1. how Docker images are to be built
2. how to run tests in Docker containers
3. how to run the local Grapl environment

The Makefile references Docker Compose files for each target that uses Docker (most of them).

### Building images

We use Dockerfile [multi-stage
builds](https://docs.docker.com/develop/develop-images/multistage-build/) so
each service can be built with a single Docker build command. Additionally, we
use [docker buildx
bake](https://github.com/docker/buildx#buildx-bake-options-target) to build
multiple Docker images with a single BuildKit command, which allows us to
leverage BuildKit concurrency across all stages. The Docker build arguments for
each service and container are defined in various Docker Compose files.

For exmaple, to build Grapl services we have the following Make target:

```Makefile
DOCKER_BUILDX_BAKE := docker buildx bake $(DOCKER_BUILDX_BAKE_OPTS)

...

.PHONY: build-services
build-services: ## Build Grapl services
	$(DOCKER_BUILDX_BAKE) -f docker-compose.yml
```

Within [docker-compose.yml](./docker-compose.yml), we have various services
such as the Sysmon generator. The following defines how to build the Docker
image.

```yaml
  grapl-rust-sysmon-subgraph-generator:
    image: grapl/grapl-sysmon-subgraph-generator:${TAG:-latest}
    build:
      context: src/rust
      target: sysmon-subgraph-generator
      args:
        - CARGO_PROFILE=${CARGO_PROFILE:-debug}
...
```

Similar can be seen for other Grapl services, as well as Grapl tests, which can
be found under the `test` directory.

### Running tests

Most Grapl Dockerfiles have build targets specific for running tests, which

Docker Compose is used to define the containers for running tests, as well as
the how the image for the container should be built. The following is the
definition for Rust unit tests
([test/docker-compose.unit-tests-rust.yml](./test/docker-compose.unit-tests-rust.yml)):

```yaml
version: "3.8"

# environment variable PWD is assumed to be grapl root directory

services:

  grapl-rust-test:
    image: grapl/rust-test-unit:${TAG:-latest}
    build:
      context: ${PWD}/src/rust
      target: build-test-unit
      args:
        - CARGO_PROFILE=debug
    command: cargo test
```

The `build-test-unit` target is a [Dockerfile](./src/rust/Dockerfile) stage
that will builds dependencies for `cargo test` that wasn't done so in the
initial source build, `cargo build`.

We're currently using `docker-compose up` to run our tests concurrently. We
have a [helper script](./test/docker-compose-with-error.sh) that checks the
exit code for each container (test) run. If any test exit code is non-zero, the
script will return non-zero as well. This allows us to surface non-zero exit
codes to Make.

## Running your locally-built Grapl images

The `make up` command will build Grapl sources and launch Docker Compose to run
the Grapl environment locally.

If you'd like to skip building and run the Grapl environment locally you can run:

``` bash
TAG=latest docker-compose up
```

Note that `TAG` should be set to whatever you used in your `make`
invocation (see [previous section](#building-the-source)).

Alternatively, you can set tag to of the tags to a particular Grapl release we
have posted on our Dockerhub. At the time of this writing there are no releases
currently supported for local Grapl, however the `main` tag is kept
up-to-date with the latest `main` branch on GitHub for development and
testing. Example:

``` bash
TAG=main docker-compose up
```

## The CI system

We use [Github Actions](https://github.com/features/actions) for
automated builds, automated tests, and automated releases. There are
three workflow definitions:

  - [grapl-lint.yml](./.github/workflows/grapl-lint.yml) -- This
    workflow runs on every PR, and every time a PR is updated. It
    makes sure our Python and Rust sources are formatted properly, and
    that Python versions have been bumped (e.g. that Python artifacts
    can be pushed to PyPI).
  - [grapl-build.yml](./.github/workflows/grapl-build.yml) -- This
    workflow also runs on every PR and every PR update. It runs all
    build and test targets, and performs some additional
    analysis on the codebase (e.g. [LGTM](https://lgtm.com/) checks).
  - [grapl-release.yml](./github/workflows/grapl-release.yml) -- This
    workflow runs every time we cut a [Github
    Release](https://github.com/grapl-security/grapl/releases). It
    builds all the release artifacts, runs all the tests, publishes
    all the Grapl images to Dockerhub so folks can run local Grapl
    easily and publishes Python libraries to PyPI.
  - [cargo-audit.yml](./github/workflows/cargo-audit.yml) -- Runs [cargo
    audit](https://github.com/RustSec/cargo-audit) to check our Rust
    dependencies for security vulnerabilities on every PR when Rust dependencies
    are changed. Also periodically runs over _all_ Rust dependencies unconditionally.

## A philosophical note

The core values of Grapl's build system are:

  - Simplicity -- It should be easy to understand what everything
    does, and why. You need only to remember one thing: `make help`.
  - Evolvability -- It should be easy to add functionality. When
    adding a new Grapl service or library to the build system you just
    need to update a Dockerfile, and corresponding Docker Compose files.
  - Orthogonality -- All the tools should be easily composed. For
    example, in each of Grapl's source subtrees you will find that we
    use the normal build tools for each language. So in `src/rust` you
    can execute `cargo test` to run all the Rust tests. In
    `src/python/*` you can run `py.test` to execute python tests. We
    run these same commands in the build system.

Note that this list *does not include* the following:

  - Cleverness -- Clever is complex. Clever is exhausting. Clever is
    hard to work with. We use the normal tools in the normal way. No
    clever hacks.
  - Innovation -- Innovation is expensive. We strive to minimize
    innovation, and constrain it to only those areas where it's
    *necessary*. Reinventing things that have already been done better
    elsewhere drains value instead of adding it.
