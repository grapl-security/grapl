# Contributing to Grapl

Thank you for your interest in contributing to Grapl! Community
contributions such as yours are one of the key factors which
differentiate Grapl in the DFIR ecosystem. This document is intended
as a guide to help you contribute as effectively as possible.

First, please read our [Code of Conduct](CODE_OF_CONDUCT.md). We
expect all contributors to follow it.

## Types of contributions

Here are some of the ways you can contribute to Grapl. All
contributions are welcome, this is not meant to be an exhaustive list.

  - Feature requests or bug reports
  - Documentation
  - Plugins
  - Core features

### Feature requests or bug reports

First, if you believe that you have identified a security
vulnerability which affects Grapl users, please do not open a public
GitHub issue. Instead, please contact us via email at
[security-reports@graplsecurity.com](mailto:security-reports@graplsecurity.com),
and we will work with you to resolve it safely and quickly.

For all non-sensitive feature request or bug reports, please open a
[Bug report GitHub
Issue](https://github.com/grapl-security/grapl/issues/new?template=bug_report.md)
containing a detailed, reproducible description of the problem. It may
be useful to discuss the issue before opening one in GitHub, if you'd
like you may reach out via Slack at the [Grapl slack channel (Click
for
invite)](https://join.slack.com/t/grapl-dfir/shared_invite/zt-armk3shf-nuY19fQQuUnYk~dHltUPCw),
but this is not a requirement.

### Changes to code or documentation

Grapl is organized as a monorepo with the following structure:

``` text
/grapl
├──etc
│  ├──images
│  ├──local_grapl
│  └──sample_data
└──src
   ├──js
   │  └──{ js services …
   ├──python
   │  └──{ python services …
   └──rust
      └──{ rust services …
```

We use [GNU Make](https://www.gnu.org/software/make/),
[Docker](https://docs.docker.com/) (version 20.10 or later) and
[docker-compose](https://docs.docker.com/compose/) for local development. To
execute a Grapl debug build, execute the following command in the project root:

``` bash
make build
```

You can see all the Make targets available by running `make help` in
Grapl root. For example:

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

See [BUILDING.md](BUILDING.md) for a more in-depth description of
Grapl's build system.

To run your images locally, execute the following command in the
project root (after building):

``` bash
TAG=latest docker-compose up
```

Alerternatively, you can use `make up` to rebuild the sources and launch `docker-compoes up`.

Note that the `TAG=latest` is redundant, it's shown here because if
you specified a different tag in your build step above you would want
to specify that same tag here.

See [these
docs](https://grapl.readthedocs.io/en/latest/setup/local.html#local-grapl)
for a more in-depth guide to operating Grapl locally.

We heartily welcome code contributions, but we request for the sake of
planning and project management that each contribution be associated
with an open issue. Please communicate with us before starting
development so we can make sure everyone is on the same page and avoid
wasting time and effort.

Pull requests should be made against the `staging` branch. We will
deploy and test all changes in our staging environment before merging
them into `master`.

#### Documentation

Documentation is definitely a work-in-progress at this point. In the
likely event you find it lacking, please open a [Documentation request
GitHub
Issue](https://github.com/grapl-security/grapl/issues/new?template=documentation_request.md).
For minor edits like spelling, correctness, grammar, etc., don't worry
about opening an issue, just submit a PR.

#### Plugins

Grapl aims to provide a rich plugin ecosystem to integrate with
external systems. If you have any suggestions for new plugin
integrations please open a [Plugin request GitHub
Issue](https://github.com/grapl-security/grapl/issues/new?template=plugin_request.md).

#### Core features

For updates to Grapl's core functionality please open a [Feature request GitHub
Issue](https://github.com/grapl-security/grapl/issues/new?template=feature_request.md).

#### Other issues

For any issue not covered in the rest of this document, please feel
free to open a [blank GitHub
Issue](https://github.com/grapl-security/grapl/issues/new?template=blank_issue.md). Please
include as much information as you can to help us address it.
