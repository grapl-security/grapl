# Provisioning Dgraph

Once the CDK deploy is complete you'll need to perform some additional
manual configuration to spin up the Dgraph cluster. The CDK deploy in
the previous section has created an Autoscaling Group for the EC2
Instances where we'll run the Docker Swarm cluster. It only remains to
spin up the EC2 instances, provision a Docker Swarm cluster, and
install Dgraph on it.

To provision Dgraph:

1. Navigate to the [AWS Autoscaling
   console](https://console.aws.amazon.com/ec2autoscaling) and click
   on the Swarm Autoscaling group. Click *Edit* in the *Group Details*
   pane and set *Desired capacity*, *Minimum capacity*, and *Maximum
   capacity* all to 0. Wait for the cluster to scale down to zero
   instances. Then set *Desired capacity*, *Minimum capacity*, and
   *Maximum capacity* all to 3. Wait for the cluster to scale up to 3
   instances.

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
   `src/js/grapl-cdk/bin/deployment_parameters.ts`. This script will
   output logs to the console indicating which instance is the swarm
   manager. It will also output logs containing the hostname of each
   swarm instance.  You will need these in subsequent steps.

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
   aws s3 cp s3://${GRAPL_DEPLOY_NAME,,}-dgraph-config-bucket/docker-compose-dgraph.yml .
   aws s3 cp s3://${GRAPL_DEPLOY_NAME,,}-dgraph-config-bucket/envoy.yaml .
   ```
   where `<deployName>` is the same `deployName` you configured above
   in `bin/deployment_parameters.ts`.
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

## DGraph operations

You can manage the DGraph cluster with the docker swarm tooling by
logging into the swarm manager with SSM. If you forget which instance
is the swarm manager, you can find it using the EC2 instance tag
`grapl-swarm-role=swarm-manager`.

