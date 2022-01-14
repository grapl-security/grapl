## Pulumi Stack Configuration Values

The following configuration values must be specified in a Pulumi stack
configuration file.

### `artifacts`

Not required for local-grapl.

A mapping specifying the tagged Docker image to use for each service. See
`Pulumi.testing.yaml` in `origin/rc` for an example.

### `container-repository`

Not required for local-grapl.

The repository from which to pull container images from.

This will be different for different stacks; we promote packages through a
series of different registries that mirrors the progress of code through our
pipelines.

The value will be something like `docker.cloudsmith.io/grapl/testing`; to target
a specific image in client code, you would append the image name and tag to the
return value of this function.

Not specifying a repository will result in local images being used.

```sh
pulumi config set container-repository docker.cloudsmith.io/grapl/testing
```

### `postgres-instance-type`

Not required for local-grapl.

The RDS instance type to use for Postgres. Make sure to size it properly
according to your anticipated workload.

```sh
# A reasonable default:
# Burstable, 2GB memory, Gravitron 2.
# With 5GB of storage and all-day usage, it comes out to about $24/mo
pulumi config set postgres-instance-type db.t4g.small
```

### `postgres-version`

Not required for local-grapl.

Which version of Postgres to use. Must be >= 13.4.

```sh
pulumi config set postgres-version 13.4
```

### `networking-server-stack`

The fully-qualified name of the Pulumi stack that set up the networking that
this Nomad server cluster will be connected to. A
[stack reference](https://www.pulumi.com/docs/intro/concepts/stack/#stackreferences)
will be used to extract information about the Consul deployment in order to
complete the setup of this cluster.

```sh
pulumi config set networking-server-stack grapl/networking/testing
```

### `nomad-server-stack`

The fully-qualified name of the Pulumi stack that set up the Nomad server
cluster that this Nomad agent cluster will be connected to. A
[stack reference](https://www.pulumi.com/docs/intro/concepts/stack/#stackreferences)
will be used to extract information about the Nomad server deployment in order
to complete the setup of this cluster.

```sh
pulumi config set nomad-server-stack grapl/nomad/testing
```
