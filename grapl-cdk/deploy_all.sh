#!/usr/bin/env bash
npm run build &&
cdk deploy grapl-event-emitters-stack && \
cdk deploy graplvpcs-stack && \
cdk deploy graplhistorydb-stack && \
cdk deploy mastergraphcluster-stack && \
cdk deploy engagementgraphcluster-stack && \
cdk deploy grapl-generic-subgraph-generator-stack && \
cdk deploy grapl-sysmon-subgraph-generator-stack && \
cdk deploy grapl-node-identity-mapper-stack && \
cdk deploy grapl-node-identifier-stack && \
cdk deploy grapl-graph-merger-stack && \
cdk deploy grapl-analyzer-dispatcher-stack && \
cdk deploy grapl-analyzer-executor-stack && \
cdk deploy grapl-engagement-creator-stack && \
cdk deploy grapl-engagement-creator-stack;

# cdk deploy engagement-creation-service-stack

date
