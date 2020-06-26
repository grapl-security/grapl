#!/usr/bin/env bash
npm run build &&
cdk deploy --require-approval=never Grapl && \
rm -rf ./edge_ux_package && \
cdk synth && \
cdk deploy --require-approval=never EngagementUX && \

date
