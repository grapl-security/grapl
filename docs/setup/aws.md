# AWS setup

**NOTE that setting up Grapl *will* incur AWS charges! This can amount to hundreds of dollars a month based on the configuration.**
This setup script is designed for testing, and may include breaking changes in future versions, increased charges in future versions, or may otherwise require manually working with CloudFormation.
If you need a way to set up Grapl in a stable, forwards compatible manner, please get in contact with us directly.

## Installing Dependencies

To get started, you'll need to install the following dependencies:

- Node
- Typescript
- AWS CDK: `npm i -g aws-cdk@X.Y.Z`
  - version must be >= the version in [Grapl's package.json file](https://github.com/grapl-security/grapl/blob/main/src/js/grapl-cdk/package.json) - for instance, `@1.71.0`
- AWS CLI: `pip install awscli`

You'll also need to have local AWS credentials, and a configuration profile. Instructions [here](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html).

If you intend to use Grapl's provided demo data, you'll allso need some Python3 dependencies.
- [boto3](https://github.com/boto/boto3)
- [zstd](https://pypi.org/project/zstd/)


## Clone from repo

First things first, clone the repo:
```bash
git clone https://github.com/grapl-security/grapl.git
```

## Build deployment artifacts

To execute a local Grapl build, run the following in Grapl's root:

```bash
TAG=$GRAPL_VERSION make zip
```

`GRAPL_VERSION` can be any name you want. Just make note of it, we'll
use it in the next step.

Alternatively, you can set TAG in a file named `.env` in the Grapl root directory. Ex:

```bash
$ cat .env
TAG="v0.0.1-example"
$ make zip
```

Similar can be done for the environment variables corresponding to CDK
deployment parameters documented in the following section.

Your build outputs should appear in the `src/js/grapl-cdk/zips/` directory.

## Configure

There are a few CDK deployment parameters you'll need to configure before you can deploy.
Each of these can be found in `bin/deployment_parameters.ts`:

1. `GRAPL_DEPLOYMENT_NAME` (required)

    Name for the deployment to AWS. We recommend prefixing the
    deployment name with "Grapl-" to help identify Grapl resources in
    your AWS account, however this isn't necessary.

    Note: This name must be globally (AWS) unique, as names for AWS S3
    buckets will be dervied from this.

2. `GRAPL_VERSION`

    The version of Grapl to deploy. This string will be used to look
    for the appropriate filenames in the `zips/` directory.

    Defaults to `latest`.

3. `GRAPL_CDK_WATCHFUL_EMAIL` (optional)

    Setting this enables [Watchful](https://github.com/eladb/cdk-watchful) for
    monitoring Grapl with email alerts.

4. `GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL` (optional)

    Setting this enables alarms meant for the operator of the Grapl stack.

5. `GRAPL_CDK_SECURITY_ALARMS_EMAIL` (optional)

    Setting this enables alarms meant for the consumer of the Grapl
    stack, for example, "a new risk node has been found".

When deploying to production we recommend creating a `source`-able 
collection of these environment variables, and saving that to some
version control; something along the lines of the following:
```bash
export GRAPL_ROOT="~/src/grapl"
export GRAPL_DEPLOYMENT_NAME="some-grapl-deployment-name"
export GRAPL_VERSION="latest"
export GRAPL_REGION="us-west-2"
export GRAPL_CDK_WATCHFUL_EMAIL="someone+watchful@domain.com"
export GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL="someone+alarm@domain.com"
export GRAPL_CDK_SECURITY_ALARMS_EMAIL="someone+alarm@domain.com"
export GRAPL_DGRAPH_INSTANCE_TYPE='i3.large'
```

## Install graplctl

We use the `graplctl` utility to deploy Grapl to AWS. To install
`graplctl` run the following command in the Grapl checkout root:

``` bash
make graplctl
```

This will build the `graplctl` binary and install it in the `bin`
directory. You can familiarize yourself with `graplctl` by running

``` bash
bin/graplctl --help
```

Note that you may wish to add some more variables to your `.env`
(mentioned above). In particular, `GRAPL_REGION`,
`GRAPL_DEPLOYMENT_NAME`, `GRAPL_VERSION`, and `AWS_PROFILE` (if not
`default`) may prove useful. If you'd rather not create an `.env` then
you may supply these values as environment/shell variables or via
command line arguments.

## Deploy Grapl

*Note: these commands spin up infrastructure in your AWS
account. Running these commands will incur charges.*

To deploy Grapl with `graplctl`, execute the following from the Grapl
root:

```bash
bin/graplctl aws deploy --all --dgraph-instance-type i3.large .
```

Note that we've selected `i3.large` instances for our DGraph
database. If you'd like to choose a different instance class, you may
see the available options by running:

``` bash
bin/graplctl aws deploy --help
```

### Provision Grapl

After we deploy to AWS successfully, we need to provision Grapl by running `./bin/graplctl aws provision` which will invoke the provisioner lambda.

## DGraph operations

You can manage the DGraph cluster with the docker swarm tooling by
logging into one of the swarm managers with SSM. If you forget which
instances are the swarm managers, you can find them by running
`graplctl swarm managers`. For your convenience, `graplctl` also
provides an `exec` command you can use to run a bash command remotely
on a swarm manager. For example, to list all the nodes in the Dgraph
swarm you can run something like the following:

``` bash
bin/graplctl swarm exec --swarm-id my-swarm-id -- docker node ls
```

If you forget which `swarm-id` is associated with your Dgraph cluster,
you may list all the swarm IDs in your deployment by running `graplctl
swarm ls`.

### Demo Data

You can send some test data up to the service by going to the root of
the grapl repo and calling:

```bash
cd $GRAPL_ROOT

# upload analyzers
etc/aws/upload_analyzer_prod.sh

# upload logs
AWS_REGION=$GRAPL_REGION \
python3 etc/local_grapl/bin/upload-sysmon-logs.py \
  --deployment_name $GRAPL_DEPLOYMENT_NAME \
  --logfile etc/sample_data/eventlog.xml
```

You can then view the progress of this data flowing through your
deployment by looking at the Cloudwatch Dashboard named
`{GRAPL_DEPLOYMENT_NAME}-PipelineDashboard`.

### Accessing the Grapl UX (Engagement Edge)

You can find the base url in `src/js/grapl-cdk/cdk-output.json`; just
append a `/index.html` to the URL in that file.
