# Debugging Tooling

Grapl has several tools to aid in debugging issues.

- Grapl Debugger Container
- Distributed Tracing

## Grapl Debugger Container

This is a container that can attach to running containers and includes debug
tools inside. See the
[debugger docs](https://github.com/grapl-security/grapl/blob/main/debugger/README.md)
for details.

## Distributed Tracing

We currently have tracing enabled for local grapl. Our current backend is
[Jaeger](https://www.jaegertracing.io/)

### Usage

1. Run `make up`
2. In your browser go to `http://localhost:16686`. You should see the Jaeger
   front-end.
3. On the left side, are search options.
4. Select the service you're interested in and click search. You can also use
   any of the additional filters on the left (such as http code, etc). If your
   service does not appear, it's possible that a) it doesn't have any traffic
   (ie the web ui needs a web request), b) there are no traces from within the
   Lookback window if
5. In the center a list of traces will appear.
6. Click on one. You will go to a page with detailed trace information,
   including performance data for each span.
7. Click on a span, and then click on tags to get more detailed information,
   such as http code, etc
8. On the top-right, there is a drop-down menu with Trace Timeline selected.
   Clicking on it will provide a few additional options

### Tracing docker buildx bake

docker buildx supports sending traces to a backend. Since we build prior to
running Jaeger, you will need to explicitly set Jaeger up, either via running
`make up` first or by running it manually in either docker or as a standalone
nomad job.

This tracing is meant to help debug docker build issues including performance
issues.

**Warning!**

1. This generates a _LOT_ of traces, enough to potentially crash Jaeger.
2. This slows down the build process _significantly_ :(

To run tracing for the docker build:

1. Do a one-time setup `make setup-docker-tracing` if you haven't already.
2. Run `WITH_TRACING=1 make build-local-infrastructure` or any other build
   command that uses bake such as `build-test-e2e`. Alternatively, you can run
   traces for individual services via
   `docker buildx bake --file $FILE --builder=grapl-tracing-builder`
