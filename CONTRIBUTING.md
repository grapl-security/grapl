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

We use [docker-compose](https://docs.docker.com/compose/) and
[dobi](https://dnephin.github.io/dobi/) for local development. To
execute a Grapl debug build, execute the following command in the
project root:

``` bash
TAG=latest GRAPL_RELEASE_TARGET=debug dobi --no-bind-mount build
```

Note that the `GRAPL_RELEASE_TARGET=debug` is redundant, but it's
shown here because if you want to execute a release build you can use
`GRAPL_RELEASE_TARGET=release`.

You can see all the `dobi` tasks available by running `dobi list` in
Grapl root. For example:

``` bash
(venv) jgrillo@penguin:~/src/grapl$ dobi list
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

See [BUILDING.md](BUILDING.md) for a more in-depth description of
Grapl's build system.

To run your images locally, execute the following command in the
project root (after building):

``` bash
TAG=latest docker-compose up
```

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
