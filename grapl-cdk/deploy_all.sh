#!/usr/bin/env bash
npm run build &&
cdk deploy --require-approval=never grapl-event-emitters-stack && \
cdk deploy --require-approval=never graplvpcs-stack && \
cdk deploy --require-approval=never graplhistorydb-stack && \
cdk deploy --require-approval=never mastergraphcluster-stack && \
cdk deploy --require-approval=never engagementgraphcluster-stack && \
cdk deploy --require-approval=never grapl-generic-subgraph-generator-stack && \
cdk deploy --require-approval=never grapl-sysmon-subgraph-generator-stack && \
cdk deploy --require-approval=never grapl-node-identity-mapper-stack && \
cdk deploy --require-approval=never grapl-node-identifier-stack && \
cdk deploy --require-approval=never grapl-graph-merger-stack && \
cdk deploy --require-approval=never grapl-analyzer-dispatcher-stack && \
cdk deploy --require-approval=never grapl-analyzer-executor-stack && \
cdk deploy --require-approval=never grapl-engagement-creator-stack && \
cdk deploy --require-approval=never grapl-user-auth-table-stack && \
cdk deploy --require-approval=never engagementedge-stack && \
cdk deploy --require-approval=never engagement-ux-stack && \
cdk deploy --require-approval=never engagements-notebook-stack && \


# cdk deploy engagement-creation-service-stack

date
