# AWS setup

## Warnings

**NOTE that setting up Grapl *will* incur AWS charges! This can amount to hundreds of dollars a month based on the configuration.**
This setup script is designed for testing, and may include breaking changes in future versions, increased charges in future versions, or may otherwise require manually working with CloudFormation.
If you need a way to set up Grapl in a stable, forwards compatible manner, please get in contact with us directly.

## Preparation

### Local AWS credentials

See full instructions [here](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html).

You should have a local file `~/.aws/credentials`, with an entry resembling this format:

```
[my_profile]
aws_access_key_id=...
aws_secret_access_key=...
aws_session_token=...
```

You will need the **profile** to configure your account, if you haven't already:

`aws configure --profile "my_profile"`

If your profile's name is not "default", then note it down, as you will need to include it as a parameter in later steps.

### Installing Dependencies

You'll need to have the following dependencies installed:

- Node
- TypeScript
- AWS CDK:
  - `npm i -g aws-cdk@X.Y.Z`
  - version must be >= the version in [Grapl's package.json file](https://github.com/grapl-security/grapl/blob/main/src/js/grapl-cdk/package.json) - for instance, `@1.71.0`
- AWS CLI:
  - your choice of the following:
    - `pip install awscli`
    - https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2-docker.html
      - helpful alias: `alias aws='docker run --rm -it -v ~/.aws:/root/.aws -v $(pwd):/aws -e AWS_PROFILE amazon/aws-cli'`
- (optional) Python libraries for running Grapl demo ([boto3](https://github.com/boto/boto3), [zstd](https://pypi.org/project/zstd/)):
  - `pip install boto3 zstd`

### Clone Grapl Git repository

```bash
git clone https://github.com/grapl-security/grapl.git
cd grapl/
```

The remaining steps assume your working directory is the Grapl repository.

### Build deployment artifacts

Deployment artifacts are build via `make zip`.

If environmental variable `TAG` is set, it will be a custom name for the build.  If it is unset, it will default to "latest".

```bash
# build with 'latest' TAG:
make zip

# build with a one-off custom TAG:
$ TAG=my_grapl_test make zip

# or set a fixed custom TAG in .env:
$ cat .env
TAG="my_grapl_test"
$ make zip
```

After `make zip` finishes, you can inspect `src/js/grapl-cdk/zips/` to see the build outputs:

```bash
ls ./src/js/grapl-cdk/zips/
```

They should be named according to the value of `TAG`.

### Configure CDK deployment parameters

Grapl's CDK deployment parameters are set as environmental variables.

For a direct code reference, c.f. `src/js/grapl-cdk/bin/deployment_parameters.ts`.

#### Recommended approach

If you are deploying to production, we recommend saving a `source`-able
collection of these environment variables:

```bash
# example values
export DEPLOYMENT_NAME="grapl-deployment-name"
export GRAPL_VERSION="latest" # if you set TAG, update this too
export GRAPL_ROOT="/path/to/grapl_git_repository"
export GRAPL_REGION="us-xxxx-n"
export GRAPL_CDK_WATCHFUL_EMAIL="email-for-watchful@example.com"
export GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL="email-for-op-alarm@example.com"
export GRAPL_CDK_SECURITY_ALARMS_EMAIL="email-for-sec-alarm@example.com"
export GRAPL_DGRAPH_INSTANCE_TYPE='xn.size' # e.g., 'i3.large'
```
#### Parameter explanation

1. `DEPLOYMENT_NAME` (required)

    A name for the deployment to AWS.  ([AWS naming requirements](https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-s3-bucket-naming-requirements.html) apply)

    This name must be globally (AWS) unique, as names for AWS S3 buckets will be dervied from this.

    We recommend prefixing the deployment name with "Grapl-" to help identify Grapl resources in your AWS account.

2. `GRAPL_VERSION` (required, if TAG changed)

    Which locally built version of Grapl to deploy.

    This string will be used to identify build outputs by their filenames in the `src/js/grapl-cdk/zips/` directory.

    If you changed `TAG` earlier, you must set this to be that same value.

    Otherwise, you may safely let it default to 'latest'.

3. `GRAPL_ROOT` (required)

    This is the path to the directory where you checked Grapl out as a Git repository.

4. `GRAPL_REGION` (required)

    This is your [AWS region](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html).

5. `GRAPL_CDK_WATCHFUL_EMAIL` (optional)

    Setting this enables [Watchful](https://github.com/eladb/cdk-watchful) for monitoring Grapl with email alerts.

6. `GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL` (required)

    Setting this enables alarms meant for the operator of the Grapl stack.

7. `GRAPL_CDK_SECURITY_ALARMS_EMAIL` (required)

    Setting this enables alarms meant for the consumer of the Grapl stack, for example, "a new risk node has been found".

## `graplctl`

### Installation

We use the `graplctl` utility to deploy Grapl to AWS. To install
`graplctl` run the following command in the Grapl checkout root:

``` bash
make graplctl
```

This will build the `graplctl` binary and install it in the `./bin/`
directory. You can familiarize yourself with `graplctl` by running

``` bash
./bin/graplctl --help
```

#### Usage notes for setup

If your AWS profile is not named 'default', you will need to explicitly provide it as a parameter:

- as a command line invocation parameter
- as an environmenal variable
- or as an entry in `.env`

### How to deploy Grapl

*Warning: these commands spin up infrastructure in your AWS
account. Running these commands will incur charges.*

To deploy Grapl with `graplctl`, execute the following from the Grapl root:

```bash
./bin/graplctl aws deploy --all --dgraph-instance-type i3.large .
```

Note that we've selected `i3.large` instances for our DGraph
database. If you'd like to choose a different instance class, you may
see the available options by running:

``` bash
bin/graplctl aws deploy --help
```

### Provision Grapl

After we deploy to AWS successfully, we need to provision Grapl by executing the following from the root of Grapl:
```bash
./bin/graplctl aws provision
```
which will invoke the provisioner lambda.

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
you may list all the swarm IDs in your deployment by running `bin/graplctl
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
  --deployment_name $DEPLOYMENT_NAME \
  --logfile etc/sample_data/eventlog.xml
```

You can then view the progress of this data flowing through your
deployment by looking at the Cloudwatch Dashboard named
`{DEPLOYMENT_NAME}-PipelineDashboard`.

### Accessing the Grapl UX (Engagement Edge)

You can find the base url in `src/js/grapl-cdk/cdk-output.json`; just
append a `/index.html` to the URL in that file.

### Logging In To Grapl

To login to Grapl, your username will be your deployment name followed by `-grapl-test-user`. For example, if your deployment was named `test-deployment`, your username would be `test-deployment-grapl-test-user`.

To retrieve the password for your grapl deployment, navigate to "AWS Secrets Manager" and click on "Secrets".

Click on the "Secret name" url that represents your deployment name followed by `-TestUserPassword`. The link will bring you to the "secret details" screen. Scroll down to the section labeled "Secret Value" and click the "Retrieve Secret Value" button. The password for your deployment will appear under "Plaintext".
