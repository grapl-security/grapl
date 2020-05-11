#!/usr/bin/env bash
npm run build &&
cdk destroy -f --require-approval=never grapl-event-emitters-stack && \
cdk destroy -f --require-approval=never graplvpcs-stack && \
cdk destroy -f --require-approval=never graplhistorydb-stack && \
cdk destroy -f --require-approval=never mastergraphcluster-stack && \
cdk destroy -f --require-approval=never engagementgraphcluster-stack && \
cdk destroy -f --require-approval=never grapl-generic-subgraph-generator-stack && \
cdk destroy -f --require-approval=never grapl-sysmon-subgraph-generator-stack && \
cdk destroy -f --require-approval=never grapl-node-identity-mapper-stack && \
cdk destroy -f --require-approval=never grapl-node-identifier-stack && \
cdk destroy -f --require-approval=never grapl-graph-merger-stack && \
cdk destroy -f --require-approval=never grapl-analyzer-dispatcher-stack && \
cdk destroy -f --require-approval=never grapl-analyzer-executor-stack && \
cdk destroy -f --require-approval=never grapl-engagement-creator-stack && \
cdk destroy -f --require-approval=never grapl-user-auth-table-stack && \
cdk destroy -f --require-approval=never engagementedge-stack && \
cdk destroy -f --require-approval=never engagement-ux-stack && \
cdk destroy -f --require-approval=never engagements-notebook-stack && \


# cdk deploy engagement-creation-service-stack

date


