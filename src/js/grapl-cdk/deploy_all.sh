#!/usr/bin/env bash
npm run build &&
cdk deploy --require-approval=never --outputs-file=./cdk-output.json Grapl && \
rm -rf ./edge_ux_package && \
npm run create_edge_ux_package && \
cdk synth && \
cdk deploy --require-approval=never EngagementUX && \

date
