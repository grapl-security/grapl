## Overview

Pulumi supports policies as part of their testing (called "CrossGuard" in
Pulumi's parlance). Policies essentially allow you to do attribute-based testing
across the entire pulumi codebase. Ie want to prevent the creation of
unencrypted ec2 instances? You would use a policy.

Pulumi supports two types of policies:

- Resource Validation
- Stack Validation

### Resource validation

- for running a policy against a single resource type
- Checked **before** resources are created/modified
- ex: checking that s3 buckets have a specific acl set

### Stack validation

- for running a policy against 2+ types of resources at the same time OR to
  validate against a resource that must already be created to validate (like ACM
  certificates)
- Checked **after** resources are created (it's essentially a post-creation
  audit)
- ex: Enforcing a tagging policy
- ex: checking that all fargate instances have an autoscaling policy set.

See https://www.pulumi.com/docs/guides/crossguard/core-concepts/ for more info
about resource and stack validation

## Folder Structure

```
policies
  3rdparty # put 3rd party policies in this folder within their own named folder
    awsguard
  grapl # place internally developed policies here

```

For internal folders we prefer policies to be written in python. For 3rd party
policy packs, they can be written in any supported language

## Testing policies

```shell
cd pulumi/policies/3rdparty/awsguard && npm install
cd ../../../grapl
pulumi preview --policy-pack ../policies/grapl/ --policy-pack ../policies/3rdparty/awsguard/
```
