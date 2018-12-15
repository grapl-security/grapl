#!/usr/bin/env bash
npm run build &&
cdk deploy vpcs-stack &&
cdk deploy event-emitters-stack &&
cdk deploy history-db-stack &&
cdk deploy node-identity-mapper-stack &&
cdk deploy generic-subgraph-generator-stack &&
cdk deploy sysmon-subgraph-generator-stack &&
cdk deploy node-identifier-stack &&
cdk deploy graph-merger-stack &&
cdk deploy engagement-creation-service-stack

