# AWS setup

## Warnings

**NOTE that setting up Grapl _will_ incur AWS charges! This can amount to
hundreds of dollars a month based on the configuration.** This setup script is
designed for testing, and may include breaking changes in future versions,
increased charges in future versions, or may otherwise require manually working
with CloudFormation. If you need a way to set up Grapl in a stable, forwards
compatible manner, please get in contact with us directly.

## Preparation

### Local AWS credentials

See full instructions
[here](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-configure.html).

You should have a local file `~/.aws/credentials`, with an entry resembling this
format:

```
[my_profile]
aws_access_key_id=...
aws_secret_access_key=...
aws_session_token=...
```

You will need the **profile** to configure your account, if you haven't already:

`aws configure --profile "my_profile"`

If your profile's name is not "default", then note it down, as you will need to
include it as a parameter in later steps.

### Installing Dependencies

You'll need to have the following dependencies installed:

- Pulumi: https://www.pulumi.com/docs/get-started/install/
- AWS CLI:
  - your choice of the following:
    - `pip install awscli`
    - https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2-docker.html
      - helpful alias:
        `alias aws='docker run --rm -it -v ~/.aws:/root/.aws -v $(pwd):/aws -e AWS_PROFILE amazon/aws-cli'`

### Clone Grapl Git repository

```bash
git clone https://github.com/grapl-security/grapl.git
cd grapl/
```

The remaining steps assume your working directory is the Grapl repository.

### Build deployment artifacts

Deployment artifacts are build via `make lambdas`.

If environmental variable `TAG` is set, it will be a custom name for the build.
If it is unset, it will default to "latest".

```bash
# build with 'latest' TAG:
make pulumi-prep

# build with a one-off custom TAG:
$ TAG=my_grapl_test make pulumi-prep

# or set a fixed custom TAG in .env:
$ cat .env
TAG="my_grapl_test"
$ make pulumi-prep
```

## Spin up infrastructure with Pulumi

See [pulumi/README.md](../../pulumi/README.md) for instructions to spin up
infrastructure in AWS with Pulumi. Once you have successfully deployed Grapl
with Pulumi, return here and follow the instructions in the following section to
provision Grapl and run the tests.

## `graplctl`

We use the `graplctl` utility to manage Grapl in AWS.

### Installation

To install `graplctl` run the following command in the Grapl checkout root:

```bash
make graplctl
```

This will build the `graplctl` binary and install it in the `./bin/` directory.
You can familiarize yourself with `graplctl` by running

```bash
./bin/graplctl --help
```

#### Usage notes for setup

If your AWS profile is not named 'default', you will need to explicitly provide
it as a parameter:

- as a command line invocation parameter
- as an environmenal variable

### How to spin up DGraph

_Warning: these commands spin up infrastructure in your AWS account. Running
these commands will incur charges._

To spin up DGraph with `graplctl`, execute the following from the Grapl root:

```bash
./bin/graplctl dgraph create --instance-type i3.large
```

Note that we've selected `i3.large` instances for our DGraph database. If you'd
like to choose a different instance class, you may see the available options by
running:

```bash
bin/graplctl aws deploy --help
```

### Provision Grapl

After we deploy to AWS successfully, we need to provision Grapl by executing the
following from the root of the Grapl repository checkout:

```bash
./bin/graplctl aws provision
```

which will invoke the provisioner lambda.

## Testing

Follow the instructions in this section to deploy analyzers, upload test data,
and execute the end-to-end tests in AWS.

### Deploy analyzers

To deploy the test analyzers, run the following `graplctl` commands:

```bash
./bin/graplctl upload analyzer --analyzer_main_py etc/local_grapl/unique_cmd_parent/main.py
```

```bash
./bin/graplctl upload analyzer --analyzer_main_py etc/local_grapl/suspicious_svchost/main.py
```

Now Grapl is ready to analyze a test dataset.

### Upload test data

To upload the test data, run the following `graplctl` command:

```bash
./bin/graplctl upload sysmon --logfile etc/sample_data/eventlog.xml
```

This will send the test dataset to the appropriate location in S3, which will
kick off the Grapl data pipeline.

### Execute the end-to-end tests

To execute the end-to-end tests, run the following `graplctl` command:

```bash
./bin/graplctl aws test
```

This will execute the `e2e-test-runner` lambda in AWS.

### Logging in to the Grapl UI with the test user

You may use the test user to log into Grapl and interact with the UI. The test
username is the deployment name followed by `-grapl-test-user`. For example, if
your deployment was named `test-deployment`, your username would be
`test-deployment-grapl-test-user`.

To retrieve the password for your grapl deployment, navigate to "AWS Secrets
Manager" and click on "Secrets".

Click on the "Secret name" url that represents your deployment name followed by
`-TestUserPassword`. The link will bring you to the "secret details" screen.
Scroll down to the section labeled "Secret Value" and click the "Retrieve Secret
Value" button. The password for your deployment will appear under "Plaintext".

## DGraph operations

You can manage the DGraph cluster with the docker swarm tooling by logging into
one of the swarm managers with SSM. If you forget which instances are the swarm
managers, you can find them by running `graplctl swarm managers`. For your
convenience, `graplctl` also provides an `exec` command you can use to run a
bash command remotely on a swarm manager. For example, to list all the nodes in
the Dgraph swarm you can run something like the following:

```bash
bin/graplctl swarm exec --swarm-id my-swarm-id -- docker node ls
```

If you forget which `swarm-id` is associated with your Dgraph cluster, you may
list all the swarm IDs in your deployment by running `bin/graplctl swarm ls`.
