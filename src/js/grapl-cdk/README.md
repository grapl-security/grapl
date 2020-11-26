# Grapl CDK

Here you will find the [AWS CDK](https://aws.amazon.com/cdk/) code for
a Grapl deployment in AWS.

## Pre-requisites

Execute these steps to prepare a Grapl CDK deployment.

### Dependencies

Install the following dependencies:

1. Node
2. Typescript
3. AWS CDK -- `npm i -g aws-cdk@1.71.0`
4. AWS CLI -- `pip install awscli`

### AWS Credentials

Make sure your `~/.aws/credentials` file contains the proper AWS
credentials.

### Grapl deployment artifacts

There are two options for obtaining deployment artifacts. One is to
execute a Grapl build locally and extract artifacts from the build
containers. Another is to download pre-built release artifacts from
Github.

#### Downloading pre-built release artifacts from Github

Navigate to https://github.com/grapl-security/grapl/releases and find
the git tag associated with the latest release. Then `cd
src/js/grapl-cdk` and run `python3 fetch_zips_by_tag.py $TAG` where
`$TAG` is the appropriate git tag. The script will download all the
release artifacts to the `zips/` directory.

#### Executing a Grapl build and extracting release artifacts

To execute a local Grapl build, run the following in Grapl's root:

```bash
TAG=$GRAPL_VERSION GRAPL_RELEASE_TARGET=release dobi --no-bind-mount build
```

Then extract the deployment artifacts from the build containers with
the following script:

```bash
VERSION=$GRAPL_VERSION ./extract-grapl-deployment-artifacts.sh
```

`GRAPL_VERSION` can be any name you want. Just make note of it, we'll
use it in the next step.

Your build outputs should appear in the `zips/` directory.

### Configuration

There are a few CDK deployment parameters to configure. Each of these
can be found in `bin/deployment_parameters.ts`:

1. `deployName` (required)

    Name for the deployment to AWS. We recommend prefixing the
    deployment name with "Grapl-" to help identify Grapl resources in
    your AWS account, however this isn't necessary.

    Note: This name must be globally (AWS) unique, as names for AWS S3
    buckets will be dervied from this.

    env: `GRAPL_CDK_DEPLOYMENT_NAME`

2. `graplVersion`

    The version of Grapl to deploy. This string will be used to look
    for the appropriate filenames in the `zips/` directory.

    Defaults to `latest`.

    env: `GRAPL_VERSION`

3. `watchfulEmail` (optional)

    Setting this enables [Watchful](https://github.com/eladb/cdk-watchful) for
    monitoring Grapl with email alerts.

    env: `GRAPL_CDK_WATCHFUL_EMAIL`

4. `operationalAlarmsEmail` (optional)

    Setting this enables alarms meant for the operator of the Grapl stack.

    env: `GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL`

5. `securityAlarmsEmail` (optional)

    Setting this enables alarms meant for the consumer of the Grapl
    stack, for example, "a new risk node has been found".

    env: `GRAPL_CDK_SECURITY_ALARMS_EMAIL`

Alternatively, these can be set via the environment variables
mentioned for each above. The environment variables take precedence
over the values in `bin/deployment_parameters.ts`.

When deploying to production we recommend *not* using environment
variables for setting parameters, but rather set them in
`bin/deployment_parameters.ts` and save the changes in a git
branch. This should help future maintenance of the deployment.

## Deploying

To deploy Grapl with the CDK, execute the following. Note that this
process takes a while (like roughly 1hr), especially the last step.

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

3. `cd src/js/grapl-cdk/swarm` and run `python3 swarm_setup.py
   $GRAPL_DEPLOY_NAME` where `$GRAPL_DEPLOY_NAME` is the same
   `deployName` you configured above in
   `src/js/grapl-cdk/bin/grapl-cdk.ts`. This script will output logs
   to the console indicating which instance is the swarm manager. It
   will also output logs containing the hostname of each swarm
   instance. You will need these in subsequent steps.

4. Navigate to the [AWS Session Manager
   console](https://console.aws.amazon.com/systems-manager/session-manager)
   and click *Start session*. Select the swarm manager instance. A
   shell session will open on that instance.

5. Execute the following commands in the SSM shell on the swarm
   manager. For your convenience, in step (3) above, the
   `swarm_setup.py` script has logged them to your terminal with the
   appropriate substitutions filled in:
   ```bash
   sudo su ec2-user
   cd $HOME

   # get DGraph configs
   GRAPL_DEPLOY_NAME=<deployName>
   aws s3 cp s3://$GRAPL_DEPLOY_NAME-dgraph-config-bucket/docker-compose-dgraph.yml .
   aws s3 cp s3://$GRAPL_DEPLOY_NAME-dgraph-config-bucket/envoy.yaml .
   ```
   where `<deployName>` is the same `deployName` you configured above
   in `bin/grapl-cdk.ts`.
   ``` bash
   export AWS_LOGS_GROUP=<log_group_name>
   export AWS01_NAME=<swarm_manager_hostname>
   export AWS02_NAME=<swarm_worker1_hostname>
   export AWS03_NAME=<swarm_worker2_hostname>

   # start DGraph
   docker stack deploy -c docker-compose-dgraph.yml dgraph

   # check that all the services are running
   docker service ls
   ```

   where `<swarm_manager_hostname>`, `<swarm_worker1_hostname>`, and
   `<swarm_worker2_hostname` are the hostnames of all the instances
   from the script logs in step (3) above
   (e.g. `ip-10-0-148-238.ec2.internal`). You can choose anything you
   want for `<log_group_name>`, it just needs to be unique in the
   region where Grapl is deployed. Therefore it's probably worthwhile
   to include `$GRAPL_DEPLOY_NAME` as a name component.

# DGraph operations

You can manage the DGraph cluster with the docker swarm tooling by
logging into the swarm manager with SSM. If you forget which instance
is the swarm manager, you can find it using the EC2 instance tag
`grapl-swarm-role=swarm-manager`.
