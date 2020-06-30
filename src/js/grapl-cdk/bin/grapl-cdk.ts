#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';

import { GraplCdkStack } from '../lib/grapl-cdk-stack';
import { EngagementUx } from '../lib/engagement';

const deployName = 'Grapl-MYDEPLOYMENT';
const graplVersion = 'latest';
//const watchfulEmail = 'YOUR@EMAIL';

const app = new cdk.App();
const grapl = new GraplCdkStack(app, 'Grapl', {
    version: graplVersion,
    stackName: deployName,
    tags: {'grapl deployment': deployName},
});

new EngagementUx(app, 'EngagementUX', {
    prefix: grapl.prefix,
    engagement_edge: grapl.engagement_edge,
    graphql_endpoint: grapl.graphql_endpoint,
    stackName: deployName + '-EngagementUX',
});
