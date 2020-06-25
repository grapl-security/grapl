# Grapl CDK

Here you will find the [AWS CDK](https://aws.amazon.com/cdk/) code for
a Grapl deployment in AWS.

## Pre-requisites

Execute these steps to prepare a Grapl CDK deployment.

### Dependencies

Install the following dependencies:
  1. Node
  2. Typescript
  3. AWS CDK -- `npm i -g aws-cdk@1.46.0`
  4. AWS CLI

### Configuration

Make sure your `~/.aws/credentials` file contains the proper AWS credentials.

### Grapl build artifacts

Execute a local Grapl build by running the following in Grapl's root:

``` bash
docker-compose -f docker-compose.yml -f docker-compose.build.yml build --build-arg release_target=release
```

Then extract the deployment artifacts from the build containers with
the following script:

``` bash
VERSION=$YOUR_VERSION CHANNEL=latest ./extract-grapl-deployment-artifacts.sh
```

Then move the deployment artifacts into the `/zips` directory:

``` bash
mv *.zip zips/
```

## Deploying

To deploy Grapl with the CDK, execute the following

  1. `npm -i`
  2. `npm run build`
  3. `echo "BUCKET_PREFIX=$YOUR_BUCKET_PREFIX" > .env`
  4. `cdk bootstrap` (only need to do this once per region)
  5. `./deploy_all.sh`
