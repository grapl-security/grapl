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
the previous section has created a bastion host which we will now use
to provision a Docker Swarm cluster and install DGraph on it. We use
AWS Secure Session Manager (SSM) to access this bastion host via the
AWS Console.

To provision DGraph:

1. Navigate to the [AWS Session Manager
   console](https://us-east-1.console.aws.amazon.com/systems-manager/session-manager)
   and click _Start session_. A new window will open in your browser
   with a terminal prompt on the bastion host.
2. Execute the following commands:

```bash
# install docker
sudo yum install -y docker
sudo systemctl enable docker.service
sudo systemctl start docker.service
sudo usermod -a -G docker ec2-user
sudo su ec2-user
cd

# The Grapl deployment name we used in the CDK
GRAPL_DEPLOYMENT=<YOUR_DEPLOYMENT>

# install docker-machine
base=https://github.com/docker/machine/releases/download/v0.16.2 &&
curl -L $base/docker-machine-$(uname -s)-$(uname -m) >/tmp/docker-machine &&
sudo mv /tmp/docker-machine /usr/local/bin/docker-machine &&
chmod +x /usr/local/bin/docker-machine
export PATH=/usr/local/bin:$PATH

# extract AWS creds into environment variables
ROLE=$(curl http://169.254.169.254/latest/meta-data/iam/security-credentials/)
RESPONSE=$(curl http://169.254.169.254/latest/meta-data/iam/security-credentials/$ROLE)
AWS_ACCESS_KEY_ID=$(echo $RESPONSE | python -c 'import json; import sys; print(json.load(sys.stdin)["AccessKeyId"]);')
AWS_SECRET_ACCESS_KEY=$(echo $RESPONSE | python -c 'import json; import sys; print(json.load(sys.stdin)["SecretAccessKey"]);')
AWS_SESSION_TOKEN=$(echo $RESPONSE | python -c 'import json; import sys; print(json.load(sys.stdin)["Token"]);')

# extract AWS region into environment variable
AWS_DEFAULT_REGION=$(curl http://169.254.169.254/latest/meta-data/placement/region)

# create a key pair
KEYPAIR_NAME=${GRAPL_DEPLOYMENT}-docker
aws --region $AWS_DEFAULT_REGION ec2 create-key-pair --key-name "$KEYPAIR_NAME" --query 'KeyMaterial' --output text > $HOME/docker-machine-key.pem
chmod 400 $HOME/docker-machine-key.pem
ssh-keygen -y -f $HOME/docker-machine-key.pem > $HOME/docker-machine-key.pem.pub

# extract security group and VPC ID into environment variables
MAC=$(curl http://169.254.169.254/latest/meta-data/network/interfaces/macs)
SWARM_SECURITY_GROUP=$(curl http://169.254.169.254/latest/meta-data/network/interfaces/macs/$MAC/security-groups)
SWARM_VPC_ID=$(curl http://169.254.169.254/latest/meta-data/network/interfaces/macs/$MAC/vpc-id)
SWARM_SUBNET_ID=$(curl http://169.254.169.254/latest/meta-data/network/interfaces/macs/$MAC/subnet-id)

# spin up ec2 resources with docker-machine
# see https://dgraph.io/docs//deploy/multi-host-setup/#cluster-setup-using-docker-swarm
EC2_INSTANCE_TYPE=t3a.medium
alias dm='/usr/local/bin/docker-machine create --driver "amazonec2" --amazonec2-private-address-only --amazonec2-vpc-id "$SWARM_VPC_ID" --amazonec2-security-group "$SWARM_SECURITY_GROUP" --amazonec2-keypair-name "$KEYPAIR_NAME" --amazonec2-ssh-keypath "$HOME/docker-machine-key.pem" --amazonec2-subnet-id "$SWARM_SUBNET_ID" --amazonec2-instance-type "$EC2_INSTANCE_TYPE" --amazonec2-region "$AWS_DEFAULT_REGION"'
export AWS01_NAME=${GRAPL_DEPLOYMENT}-aws01
export AWS02_NAME=${GRAPL_DEPLOYMENT}-aws02
export AWS03_NAME=${GRAPL_DEPLOYMENT}-aws03
dm "$AWS01_NAME"
dm "$AWS02_NAME"
dm "$AWS03_NAME"

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

# get DGraph compose template
cd $HOME
wget https://github.com/grapl-security/grapl/raw/staging/src/js/grapl-cdk/docker-compose-multi.yml

# start DGraph
eval $(docker-machine env "$AWS01_NAME" --shell sh)
docker stack deploy -c docker-compose-multi.yml dgraph

# add A records to route53 to make the alpha nodes discoverable
AWS02_IP=$(docker-machine ip "$AWS02_NAME")
AWS03_IP=$(docker-machine ip "$AWS03_NAME")
DNS_NAME=$(echo $GRAPL_DEPLOYMENT | awk '{print tolower($0)}').dgraph.grapl
HOSTED_ZONES_RESPONSE=$(aws route53 list-hosted-zones-by-name --dns-name "$DNS_NAME")
HOSTED_ZONE_ID=$(echo $HOSTED_ZONES_RESPONSE | python -c 'import json; import sys; print(json.load(sys.stdin)["HostedZones"][0]["Id"]);')
echo {\"Changes\": [{\"Action\": \"UPSERT\", \"ResourceRecordSet\": {\"Name\": \"$DNS_NAME\", \"Type\": \"A\", \"TTL\": 300, \"ResourceRecords\": [{\"Value\": \"$AWS01_IP\"}, {\"Value\": \"$AWS02_IP\"}, {\"Value\": \"$AWS03_IP\"}]}}]} > $HOME/batch.json
aws route53 change-resource-record-sets --hosted-zone-id $HOSTED_ZONE_ID --change-batch file://$HOME/batch.json

# check that all the services are running
docker service ls
```

## Operating DGraph

Now that we have DGraph provisioned, it's important to be aware of
some operational details.

First, _don't lose the key pair_. If, for example, your bastion host
crashes and you somehow lost the key pair
(e.g. `docker-machine-key-pair.pem` from the previous section) then
`docker-machine` will no longer be able to connect to the DGraph
cluster. This would be bad. To mitigate this risk, make sure you don't
destroy the bastion's EBS volume. If the bastion crashes and you need
to make a new one, be sure to use the previous bastion's EBS volume.
