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

Set your deployment name and version in `bin/grapl-cdk.ts`:

```
export const deployName = 'Grapl-MYDEPLOYMENT';
export const graplVersion = 'YOUR_VERSION';
```

Some tips for choosing a deployment name:

-   Keep "Grapl" as prefix. This isn't necessary, but will help
    identify Grapl resources in your AWS account.
-   Choose a globally unique name, as this will be used to name S3
    buckets, which have this requirement. Using a name that includes
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
the previous section has created a bastion host for cluster management
as well as an Autoscaling Group containing EC2 Instances where we'll
run the Docker Swarm cluster. It only remains to provision a Docker
Swarm cluster and install DGraph on it. We use AWS Secure Session
Manager (SSM) to access this bastion host via the AWS Console.

To provision DGraph:

1. Navigate to the [AWS Session Manager
   console](https://us-east-1.console.aws.amazon.com/systems-manager/session-manager)
   and click _Start session_. A new window will open in your browser
   with a terminal prompt on the bastion host.
2. Execute the following commands:

```bash
#
# refer to the DGraph docs for more details about the rest of the setup
# procedure:
#
# https://dgraph.io/docs//deploy/multi-host-setup/#cluster-setup-using-docker-swarm
#

# create a Docker Swarm cluster
AWS01_IP=$(docker-machine ip "$AWS01_NAME")
eval $(docker-machine env "$AWS01_NAME" --shell sh)
docker swarm init --advertise-addr $AWS01_IP

# extract the join token
WORKER_JOIN_TOKEN=$(docker swarm join-token worker -q)

# make aws02 and aws03 join the swarm
eval $(docker-machine env "$AWS02_NAME" --shell sh)
docker swarm join --token $WORKER_JOIN_TOKEN "$AWS01_IP:2377"
eval $(docker-machine env "$AWS03_NAME" --shell sh)
docker swarm join --token $WORKER_JOIN_TOKEN "$AWS01_IP:2377"

# get DGraph configs
cd $HOME
wget https://github.com/grapl-security/grapl/raw/staging/src/js/grapl-cdk/dgraph/docker-compose-dgraph.yml
wget https://github.com/grapl-security/grapl/raw/staging/src/js/grapl-cdk/dgraph/envoy.yaml

# start DGraph
eval $(docker-machine env "$AWS01_NAME" --shell sh)
docker stack deploy -c docker-compose-dgraph.yml dgraph

# check that all the services are running
docker service ls
```
