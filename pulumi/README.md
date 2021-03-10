Grapl Infrastructure
====================

Grapl is (or will soon be) deployed to AWS using [Pulumi][pulumi].

# "Local Grapl"
For testing locally, we use [Localstack][ls]. Our `local-grapl` Pulumi
stack is configured by default to point toward an instance of
Localstack running on the default port on `localhost`. After
installing Localstack, you should be able to run `localstack start`,
and then `pulumi up` against the `local-grapl` stack and deploy to
your Localstack instance. When working purely on infrastructure, this
can be a way to iterate quickly.

In our integration and testing environments, we also use Localstack,
but inside a Docker Compose network, and also currently with
[MinIO][minio] as our S3 service. This requires changing the S3
endpoint, as well as the fake AWS credentials we use for
Pulumi. However, this modification is handled within the
`docker-compose.yml` file, so it is mentioned here for informational
purposes only.

(We use MinIO currently because we have some test code that does not
appear to like Localstack's S3 implementation for an as-yet-unknown
reason. Additionally, we have had trouble configuring Localstack to
proxy MinIO as its S3 endpoint. As a result, we need to communicate
directly with MinIO on its own endpoint. Additionally, MinIO requires
a secret key of at least 8 characters, so we can't use localstack's
default value of `test`. Since we don't want the hassle of trying to
provide multiple sets of credentials to Pulumi, we just set an new set
across all our services in our Docker Compose environment. In the
future, we hope to be able to simplify this setup.)

# Getting Started

We're using the Python SDK for Pulumi. As such, we'll need to have
access to the appropriate Python libraries when we run the `pulumi`
CLI. If you have not already done so, run `make populate-venv` from
the repository root, and follow the instructions it gives you at the
end.

To run Pulumi locally, we'll need to login locally, thus avoiding
communication with the hosted Pulumi service.

```sh
pulumi login --local
```

Then, you must initialize the `local-grapl` stack on your
machine. This will create the necessary state to manage the stack, but
the configuration will be pulled from the `Pulumi.local-grapl.yaml`
file that already exists within this repository.

```sh
pulumi stack init local-grapl
```

You will be asked for a passphrase. Because this is a shared stack
only used for local and testing purposes, we can share the passphrase
as well: `local-grapl-passphrase`. Setting this in the
`PULUMI_CONFIG_PASSPHRASE` environment variable can make it easier to
interact with this stack on your machine.

[pulumi]: https://pulumi.com
[ls]: https://localstack.cloud/
[minio]: https://min.io
