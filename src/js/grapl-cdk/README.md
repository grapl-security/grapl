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

If you have configured an email address for Watchful (see previous
section) you should receive an email with subject *"AWS Notification -
Subscription Confirmation"* containing a link to activate the
subscription. Click the link to begin receiving CloudWatch alerts.

## Provisioning DGraph

Once the CDK deploy is complete you'll need to perform some additional
manual configuration to spin up the DGraph cluster. The CDK deploy in
the previous section has created an Autoscaling Group for the EC2
Instances where we'll run the Docker Swarm cluster. It only remains to
spin up the EC2 instances, provision a Docker Swarm cluster, and
install DGraph on it.

To provision DGraph:

1. Navigate to the [AWS Autoscaling
   console](https://console.aws.amazon.com/ec2autoscaling) and click
   on the Swarm Autoscaling group. Click *Edit* in the *Group Details*
   pane and set *Desired capacity*, *Minimum capacity*, and *Maximum
   capacity* all to 3.

2. Navigate to the [AWS Route53 Hosted Zones
   console](https://console.aws.amazon.com/route53/v2/hostedzones) and
   click on the hosted zone with *Domain name* ending in
   `.dgraph.grapl`. Wait until you see a DNS record of *Type* A appear
   in the list of *Records*. It may take a few minutes and you may
   have to click the refresh button. Ensure that the IP addresses
   associated with the A record are the private IP addresses of the
   instances in the Autoscaling Group from (1).

3. `cd swarm` and run `python3 swarm_setup.py $GRAPL_DEPLOY_NAME`
   where `$GRAPL_DEPLOY_NAME` is the same `deployName` you configured
   above in `bin/grapl-cdk.ts`. This script will output logs to the
   console indicating which instance is the swarm manager.

4. Navigate to the [AWS Session Manager
   console](https://us-east-1.console.aws.amazon.com/systems-manager/session-manager)
   and click *Start session*. Select the swarm manager instance. A
   shell session will open on that instance.

5. Execute the following commands in the SSM shell on the swarm
   manager:
   ```bash
   cd $HOME

   # get DGraph configs
   aws s3 cp s3://$GRAPL_DEPLOY_NAME-dgraph-config-bucket/docker-compose-dgraph.yml .
   aws s3 cp s3://$GRAPL_DEPLOY_NAME-dgraph-config-bucket/envoy.yml .
   ```
   where `$GRAPL_DEPLOY_NAME` is the same `deployName` you configured
   above in `bin/grapl-cdk.ts`.
   ``` bash
   # start DGraph
   docker stack deploy -c docker-compose-dgraph.yml dgraph

   # check that all the services are running
   docker service ls
   ```

# DGraph operations

You can manage the DGraph cluster with the docker swarm tooling by
logging into the swarm manager with SSM. If you forget which instance
is the swarm manager, you can find it using the EC2 instance tag
`grapl-swarm-role=swarm-manager`.
