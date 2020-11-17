#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';

import { GraplCdkStack } from '../lib/grapl-cdk-stack';
import { EngagementUx } from '../lib/engagement';

// Deployment parameters
const deployName = undefined; // ex: 'Grapl-my-deployment'
const graplVersion = undefined;
const watchfulEmail = undefined;

const stackName = process.env.GRAPL_CDK_DEPLOYMENT_NAME || deployName;
if (!stackName) {
    throw new Error("Error: Missing Grapl deployment name. Set via bin/grapl-cdk.ts, or environment variable GRAPL_CDK_DEPLOYMENT_NAME.");
}

const app = new cdk.App();

const grapl = new GraplCdkStack(app, 'Grapl', {
    version:process.env.GRAPL_CDK_VERSION ||  graplVersion || 'latest',
    stackName: stackName,
    watchfulEmail: process.env.GRAPL_CDK_WATCHFUL_EMAIL || watchfulEmail,
    tags: { 'grapl deployment': stackName},
    description: 'Grapl base deployment',
});

new EngagementUx(app, 'EngagementUX', {
    prefix: grapl.prefix,
    engagement_edge: grapl.engagement_edge,
    graphql_endpoint: grapl.graphql_endpoint,
    model_plugin_deployer: grapl.model_plugin_deployer,
    stackName: deployName + '-EngagementUX',
    description: 'Grapl Engagement UX',
});
