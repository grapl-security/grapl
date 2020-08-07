# Grapl CDK

Here you will find the [AWS CDK](https://aws.amazon.com/cdk/) code for
a Grapl deployment in AWS.

## Pre-requisites

Execute these steps to prepare a Grapl CDK deployment.

### Dependencies

Install the following dependencies:

1. Node
2. Typescript
3. AWS CDK -- `npm i -g aws-cdk@1.47.0`
4. AWS CLI

### AWS Credentials

Make sure your `~/.aws/credentials` file contains the proper AWS
credentials.

### Grapl build artifacts

Execute a local Grapl build by running the following in Grapl's root:

```bash
TAG=$YOUR_VERSION GRAPL_RELEASE_TARGET=release dobi --no-bind-mount build
```

Then extract the deployment artifacts from the build containers with
the following script:

```bash
VERSION=$YOUR_VERSION ./extract-grapl-deployment-artifacts.sh
```

`YOUR_VERSION` can be any name you want. Just make note of it, we'll
use it in the next step.

Your build outputs should appear in the `zips/` directory.

### Configuration

Set your deployment name and version in `bin/constants.ts`:

```
export const deployName = 'Grapl-MYDEPLOYMENT';
export const graplVersion = 'YOUR_VERSION';
```

Some tips for choosing a deployment name:

  - Keep "Grapl" as prefix. This isn't necessary, but will help
    identify Grapl resources in your AWS account.
  - Choose a globally unique name, as this will be used to name S3
    buckets, which have this requiement. Using a name that includes
    your AWS account number and deployment region should work.

To enable [Watchful](https://github.com/eladb/cdk-watchful) for
monitoring Grapl with email alerts, specify the email address to
receive alerts:

```
export const watchfulEmail = 'YOUR@EMAIL';
```

## Deploying

To deploy Grapl with the CDK, execute the following

1. `npm i`
2. `npm run build`
3. `env CDK_NEW_BOOTSTRAP=1 cdk bootstrap --cloudformation-execution-policies arn:aws:iam::aws:policy/AdministratorAccess`
4. `./deploy_all.sh`

## Provisioning DGraph

Once the CDK deploy is complete you'll need to perform some additional
manual configuration to spin up the DGraph cluster. The CDK deploy in
the previous section has created a bastion host which we will now use
to provision a Docker Swarm cluster and install DGraph on it. We use
AWS Secure Session Manager (SSM) to access this bastion host via the
AWS Console.

To provision DGraph:

  1. Navigate to the [AWS Session Manager
     console](https://us-east-1.console.aws.amazon.com/systems-manager/session-manager)
     and click *Start session*. A new window will open in your browser
     with a terminal prompt on the bastion host.
  2. `sudo yum install docker`
  3. Execute the following commands:

``` bash
# install docker-machine
base=https://github.com/docker/machine/releases/download/v0.16.0 &&
curl -L $base/docker-machine-$(uname -s)-$(uname -m) >/tmp/docker-machine &&
sudo mv /tmp/docker-machine /usr/local/bin/docker-machine &&
chmod +x /usr/local/bin/docker-machine

# extract AWS creds into environment variables
ROLE=$(curl http://169.254.169.254/latest/meta-data/iam/security-credentials/)
RESPONSE=$(curl http://169.254.169.254/latest/meta-data/iam/security-credentials/$ROLE)
AWS_ACCESS_KEY_ID=$(echo $RESPONSE | python -c 'import json; import sys; print(json.load(sys.stdin)["AccessKeyId"]);')
AWS_SECRET_ACCESS_KEY=$(echo $RESPONSE | python -c 'import json; import sys; print(json.load(sys.stdin)["SecretAccessKey"]);')
AWS_SESSION_TOKEN=$(echo $RESPONSE | python -c 'import json; import sys; print(json.load(sys.stdin)["Token"]);')

# spin up ec2 resources with docker-machine

```
