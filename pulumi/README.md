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

# Migrating from CDK

To help evaluate the faithfulness of this Pulumi port of our CDK
logic, we can run Pulumi against an existing CDK-generated Grapl
deployment in a kind of "import mode". This tells our Pulumi code to
adopt existing AWS resources into its stack state, rather than
creating them new. After the resources have been imported, we can
manage them completely through Pulumi.

If we are in "import mode", if our Pulumi code differs in any way from
the resources we are trying to import, Pulumi will tell us. We can
then inspect the difference and modify our Pulumi code appropriately.

If we are *not* in "import mode", then Pulumi will attempt to create
new resources, regardless of what exists in AWS. This is what you want
if you are creating fresh infrastructure, or interacting with
Localstack.

You *must* set this value in configuration explicitly, or the Pulumi
run *will* fail.

To enable import mode:
```sh
pulumi config set grapl:import_from_existing True
```

To disable import mode:
```sh
pulumi config set grapl:import_from_existing False
```

Once we have fully migrated away from CDK, we can remove this
configuration option and the code that supports it.

## CDK and Pulumi Configuration Caveat

If you are interacting with CDK and Pulumi at the same time (e.g.,
you're in the middle of helping migrate from one to the other), you
will need to be scrupulous about keeping your AWS credentials
up-to-date.

Pulumi can operate just fine with `aws sso login`. CDK, on the other
hand, cannot, so we have to add credentials to our
`~/.aws/credentials` file to interact with it.

If, however, your on-disk credentials are out of date, regardless of
the fact that you may have just logged in with `aws sso login`, your
Pulumi runs will hang, because it's looking at that file and getting
invalid information.

To preserve your sanity, get into the habit of updating your
credentials file regularly if you're working with both CDK and Pulumi
at the same time.

# Environment Variables
At the moment, we have a few bits of configuration we're specifying in
environment variables.

We're not using stack configuration variables because they're not
really stack-specific; they're more general.

## GRAPL_LAMBDA_ZIP_DIR

Default Value: `../src/js/grapl-cdk/zips`

This is the directory in which ZIP archives of our lambda functions
are found. If overriding, an absolute path may be used. If a relative
path is given, it must be relative to the `pulumi` directory.

## GRAPL_LAMBDA_TAG

Default Value: `latest`

Currently, our lambda ZIP archives are named as
`<LAMBDA_NAME>-<TAG>.zip`. Examples might be
"engagement-creator-v0.0.1.zip" or
"metric-forwarder-latest.zip". Importantly, all ZIP archives share the
same value for `TAG`.

[pulumi]: https://pulumi.com
[ls]: https://localstack.cloud/
