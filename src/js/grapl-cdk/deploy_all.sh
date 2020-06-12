#!/usr/bin/env bash
npm run build &&
cdk deploy --require-approval=never Grapl EngagementEdge GraphqlEndpoint && \
rm -rf ./edge_ux_package && \
cdk synth && \
cdk deploy --require-approval=never EngagementUX && \

date
