# Grapl Infrastructure

Grapl is deployed to AWS using [Pulumi][pulumi].

# Projects

We can divide our infrastructure into different Pulumi projects, each of which
can have their own individual instances ("stacks") and lifecycle.

At the moment, projects will live within their own sub-directory in the
[`pulumi`](./) directory. Our Python code also lives in this directory, under
[`infra`](./infra).

Currently we have the following projects:

## [grapl](./grapl)

This is the main definition of the Grapl service infrastructure.

The following stacks have been defined for this project:

### [local-grapl](./grapl/Pulumi.local-grapl.yaml)

This configuration is strictly for running the (subset) of Grapl that can be
supported by [Localstack][ls] in our current integration and end-to-end tests,
implemented with Docker Compose.

It is only intended to be run in "local" Pulumi mode (i.e., not connected to the
Pulumi SaaS).

### [testing](./grapl/Pulumi.testing.yaml)

This is a persistent environment, attached to Grapl's Pulumi SaaS account. It is
only to be updated in the context of Grapl's automated release pipeline.

# Getting Started

We're using the Python SDK for Pulumi. As such, we'll need to have access to the
appropriate Python libraries when we run the `pulumi` CLI. If you have not
already done so, run `make populate-venv` from the repository root, and **follow
the instructions it gives you at the end.**

To run Pulumi locally, we'll need to login locally, thus avoiding communication
with the hosted Pulumi service.

```sh
pulumi login --local
```

Then, only if you are wanting to interact directly with Localstack for some
reason, you must initialize the `local-grapl` stack on your machine. This will
create the necessary state to manage the stack, but the configuration will be
pulled from the `Pulumi.local-grapl.yaml` file that already exists within this
repository.

```sh
cd $GRAPL_ROOT/pulumi/grapl
pulumi stack init local-grapl
```

You will be asked for a passphrase. Because this is a shared stack only used for
local and testing purposes, we can share the passphrase as well:
`local-grapl-passphrase`. Setting this in the `PULUMI_CONFIG_PASSPHRASE`
environment variable can make it easier to interact with this stack on your
machine.

In general, though, it will be rare that you'll really need to do this.

# Running against AWS

If you'd like to run against your own AWS account, you should make a new stack
for this. This will be your personal stack, so make sure to set
`PULUMI_CONFIG_PASSPHRASE` accordingly.

Note that developer "sandboxes" like this are **not** currently managed by our
Pulumi SaaS account.

```sh
set -u
export STACK_NAME=<NAME>
cd $GRAPL_ROOT/pulumi/grapl
pulumi login --local
pulumi stack init "${STACK_NAME}"
pulumi config set aws:region us-east-1

# Copy some required artifacts version pins from `origin/rc`
../bin/copy_artifacts_from_rc.sh "${STACK_NAME}"

# You likely want to remove non-required artifacts from your stackfile now
vim "Pulumi.${STACK_NAME}.yaml"

# Fill in some stack values from Pulumi stacks you set up in `platform-infrastructure`
pulumi config set grapl:nomad-server-stack "grapl/nomad/$NOMAD_STACK_NAME"
pulumi config set grapl:networking-stack "grapl/networking/$NETWORKING_STACK_NAME"
```

Then, you should set your `AWS_PROFILE` in your environment, and then run
`aws sso login`.

Finally, set up an SSM tunnels to one of your Consul servers and one of your Nomad servers so you can deploy
Consul configs and Nomad jobs.

```
pulumi config set consul:address http://localhost:8500
pulumi config set nomad:address http://localhost:4646

# Do this in a separate tab, as it's not detached.
./bin/aws/ssm_consul_server.sh
./bin/aws/ssm_nomad_server.sh
```

Now, when you run `pulumi up`, you will be provisioning infrastructure in your
AWS account.

# Environment Variables

At the moment, we have a few bits of configuration we're specifying in
environment variables.

We're not using stack configuration variables because they're not really
stack-specific; they're more general.

## GRAPL_LAMBDA_ZIP_DIR

Default Value: `../dist/`

This is the directory in which ZIP archives of our lambda functions are found.
If overriding, an absolute path may be used. If a relative path is given, it
must be relative to the `pulumi` directory.

[pulumi]: https://pulumi.com
[ls]: https://localstack.cloud/
