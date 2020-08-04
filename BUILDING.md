# Building Grapl

This document describes Grapl's build system. Here you will find
instructions for building the Grapl source tree, running tests, and
running Grapl locally. This document also describes how the build
tools are used in our Github Actions based Continuous Integration (CI)
system.

## Building the source

Grapl leverages Docker to control the build environment. All Grapl
builds happen in Docker containers. This has the added benefit of
enabling Grapl developers to easily spin up the entire Grapl stack
locally for a nice interactive development experience.

To facilitate this we have Dockerfiles which describe the build and
deployment images for each service. The build system is logically
split into 3 separate parts, one for each language:

 - Rust -- All the Rust code and dependencies
 - Python -- All the Python code and dependencies
 - JS -- All the Javascript and Typescript

To actually orchestrate builds we use
[dobi](https://dnephin.github.io/dobi/config.html). Dobi allows us to
define tasks such as building the source code, running unit and
integration tests, and cleaning up build containers (e.g. in order to
re-build afresh). After [installing
dobi](https://dnephin.github.io/dobi/install.html) you can see these
tasks by running `dobi list` in the root of the Grapl source tree:

```
jgrillo@penguin:~/src/grapl$ dobi list
Resources:
  build                Build artifacts and images for all services
  clean-build          Delete all the build images
  clean-js-build       Delete the js build images
  clean-python-build   Delete the python build images
  clean-rust-build     Delete the rust build image
  integration-tests    Run all the integration tests
  js                   Build artifacts and images for js services
  js-unit-tests        Run the js unit tests
  python               Build artifacts and images for python services
  python-integration-tests Run the python integration tests
  python-unit-tests    Run the python unit tests
  rust                 Build artifacts and images for rust services
  rust-integration-tests Run the rust integration tests
  rust-unit-tests      Run the rust unit tests
  unit-tests           Run all the unit tests
```

To kick off a local build (but no tests), run the following command:

``` bash
TAG=latest GRAPL_RELEASE_TARGET=debug dobi --no-bind-mount build
```

To run all the unit tests, run the following command:

``` bash
TAG=latest GRAPL_RELEASE_TARGET=debug dobi --no-bind-mount unit-tests
```

To run all the integration tests, run the following command:

``` bash
TAG=latest GRAPL_RELEASE_TARGET=debug dobi --no-bind-mount integration-tests
```

Notice the environment variables:

  - `TAG` -- Required. This is the tag we'll use for all the Docker
    images. For local builds `latest` is fine. Production builds
    should have a specific version e.g. `v1.2.3`. Users shouldn't need
    to worry about this, our CI system takes care of it. More about
    that later.
  - `GRAPL_RELEASE_TARGET` -- Optional. This can be either `debug`
    (default), or `release`. It controls whether Rust code is compiled
    in debug or release mode.

Note also the `--no-bind-mount` option. We use a [host bind
mount](https://dnephin.github.io/dobi/config.html#mount) to emit build
artifacts to the `/dist` directory in the Grapl root:

```
jgrillo@penguin:~/src/grapl$ tree dist
dist
├── analyzer-dispatcher
├── analyzer-executor
│   └── lambda.zip
├── dgraph-ttl
│   └── lambda.zip
├── engagement-creator
│   └── lambda.zip
├── engagement-edge
│   └── lambda.zip
├── generic-subgraph-generator
├── graph-merger
├── graphql-endpoint
│   └── lambda.zip
├── model-plugin-deployer
│   └── lambda.zip
├── node-identifier
├── node-identifier-retry-handler
└── sysmon-subgraph-generator
```

### Dobi in depth

This section is a more in-depth description of our
[dobi.yaml](dobi.yaml) configuration. Dobi separates
[images](https://dnephin.github.io/dobi/config.html#image) (Docker
images) from [jobs](https://dnephin.github.io/dobi/config.html#job)
(commands to run on a Docker image). Our configuration first defines
all the images, then defines jobs which run on those images.

For example, this is how we've configured the Rust build image:

``` yaml
image=rust-build:
  image: grapl/grapl-rust-src-build
  context: src/rust
  dockerfile: Dockerfile
  args:
    release_target: "{env.GRAPL_RELEASE_TARGET:debug}"
  target: grapl-rust-src-build
  tags:
    - latest
```

And this is how the Rust build job is configured:

``` yaml
job=build-rust:
  use: rust-build
  mounts:
    - dist
  artifact:
    - ./dist/
```

Dobi also has a concept of
[aliases](https://dnephin.github.io/dobi/config.html#alias) which are
groupings of other tasks. Here is how the `rust` alias is configured:

``` yaml
alias=rust:
  tasks:
    - build-rust
    - "analyzer-dispatcher:tag"
    - "generic-subgraph-generator:tag"
    - "graph-merger:tag"
    - "node-identifier:tag"
    - "node-identifier-retry-handler:tag"
    - "sysmon-subgraph-generator:tag"
  annotations:
    description: "Build artifacts and images for rust services"
```

This task builds all the Rust sources, but does not run any unit or
integration tests. When we ran `dobi list`
[above](#building-the-source), all the aliases and their documentation
were printed to the console. Aliases therefore are the user-facing
commands, jobs--which run on images--are the internal building blocks
for these commands.

## Running your locally-built Grapl images

We use [Docker Compose](https://docs.docker.com/compose/) to manage
our local Grapl runtime environment. The
[docker-compose.yml](docker-compose.yml) file describes the
relationships between each of the Grapl services. To spin up your
locally-built Grapl, execute the following command in the Grapl root
after [installing docker-compose](https://docs.docker.com/compose/install/):

``` bash
TAG=latest docker-compose up
```

Note that `TAG` should be set to whatever you used in your `dobi`
invocation (see [previous section](#building-the-source)).

This Docker Compose environment serves double duty both for running
Grapl locally and for running integration tests in builds. To see how
that works, look for the `compose` section in [dobi.yaml](dobi.yaml).

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
    the `dobi` build and test jobs, and performs some additional
    analysis on the codebase (e.g. [LGTM](https://lgtm.com/) checks
    and [cargo-audit](https://github.com/RustSec/cargo-audit)).
  - [grapl-release.yml](./github/workflows/grapl-release.yml) -- This
    workflow runs every time we cut a [Github
    Release](https://github.com/grapl-security/grapl/releases). It
    builds all the release artifacts, runs all the tests, publishes
    all the Grapl images to Dockerhub so folks can run local Grapl
    easily, publishes Python libraries to PyPI, and attaches all the
    release artifacts to the Github Release.

## A philosophical note

The core values of Grapl's build system are:

  - Simplicity -- It should be easy to understand what everything
    does, and why. You need only to remember one thing: `dobi list`.
  - Evolvability -- It should be easy to add functionality. When
    adding a new Grapl service or library to the build system you just
    need to add (or in the case of Rust services update) a Dockerfile,
    edit `docker-compose.yml` and `dobi.yaml`, and update the
    `grapl-release.yml` action.
  - Orthogonality -- All the tools should be easily composed. For
    example, in each of Grapl's source subtrees you will find that we
    use the normal build tools for each language. So in `src/rust` you
    can execute `cargo test` to run all the Rust tests. In
    `src/python/*` you can run `py.test` to execute python tests. We
    run these same commands--via dobi jobs--in the build system.

Note that this list *does not include* the following:

  - Cleverness -- Clever is complex. Clever is exhausting. Clever is
    hard to work with. We use the normal tools in the normal way. No
    clever hacks.
  - Innovation -- Innovation is expensive. We strive to minimize
    innovation, and constrain it to only those areas where it's
    *necessary*. Reinventing things that have already been done better
    elsewhere drains value instead of adding it.
