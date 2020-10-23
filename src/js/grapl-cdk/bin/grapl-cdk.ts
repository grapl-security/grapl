#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';

import { GraplCdkStack } from '../lib/grapl-cdk-stack';
import { EngagementUx } from '../lib/engagement';

// const deployName = 'Grapl-MYDEPLOYMENT';
// const graplVersion = 'latest';
// const watchfulEmail = undefined;
const dgraphInstanceType = undefined;
const dgraphClusterSize = undefined;
const region = undefined;

const deployName = 'jgrillo-test';
const graplVersion = 'jgrillo-test';
const watchfulEmail = 'jgrillo@graplsecurity.com';

const app = new cdk.App();

const grapl = new GraplCdkStack(app, 'Grapl', {
    version: graplVersion,
    stackName: deployName,
    tags: { 'grapl deployment': deployName },
    watchfulEmail: watchfulEmail,
    dgraphInstanceType: dgraphInstanceType,
    dgraphClusterSize: dgraphClusterSize,
    description: 'Grapl base deployment',
    env: { 'region': region || process.env.CDK_DEFAULT_REGION }
});

new EngagementUx(app, 'EngagementUX', {
    prefix: grapl.prefix,
    engagement_edge: grapl.engagement_edge,
    graphql_endpoint: grapl.graphql_endpoint,
    model_plugin_deployer: grapl.model_plugin_deployer,
    stackName: deployName + '-EngagementUX',
    description: 'Grapl Engagement UX',
});
