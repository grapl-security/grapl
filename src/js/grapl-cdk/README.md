# Grapl CDK

Here you will find the [AWS CDK](https://aws.amazon.com/cdk/) code for
a Grapl deployment in AWS.

## Pre-requisites

Execute these steps to prepare a Grapl CDK deployment.

### Dependencies

Install the following dependencies:
  1. Node
  2. Typescript
  3. AWS CDK
  4. AWS CLI

### AWS Credentials

Make sure your `~/.aws/credentials` file contains the proper AWS credentials.

### Grapl build artifacts

Execute a local Grapl build by running the following in Grapl's root:

``` bash
docker-compose -f docker-compose.yml -f docker-compose.build.yml build --build-arg release_target=release
```

Then extract the deployment artifacts from the build containers with
the following script:

``` bash
VERSION=$YOUR_VERSION ./extract-grapl-deployment-artifacts.sh
```

`YOUR_VERSION` can be any name you want. Just make note of it, we'll use it in the next step.

Your build outputs should appear in the `zips/` directory.

### Configuration

Set your deployment name and version in `bin/grapl-cdk.ts`

```
const deployName = 'Grapl-MYDEPLOYMENT';
const graplVersion = 'YOUR_VERSION';
```

Some tips for choosing a deployment name:
- Keep "Grapl" as prefix. This isn't necessary, but will help identify Grapl resources in your AWS account.
- Choose a globally unique name, as this will be used to name S3 buckets, which have this requiement. Using a name that includes your AWS account number and deployment region should work.

## Deploying

To deploy Grapl with the CDK, execute the following

  1. `npm i`
  2. `npm run build`
  3. `cdk bootstrap` (only need to do this once per region)
  4. `./deploy_all.sh`
