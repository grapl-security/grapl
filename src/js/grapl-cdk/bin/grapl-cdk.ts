#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';

import { GraplCdkStack } from '../lib/grapl-cdk-stack';
import { EngagementUx } from '../lib/engagement';

// Either replace the `undefined`s or specify the environment variables.
class DeploymentParameters {
    readonly deployName = undefined  // ex: 'Grapl-my-deployment'
        || process.env.GRAPL_CDK_DEPLOYMENT_NAME; 

    readonly graplVersion = undefined 
        || process.env.GRAPL_VERSION 
        || "latest";

    readonly watchfulEmail = undefined  // (optional) ex: ops@example.com
        || process.env.GRAPL_CDK_WATCHFUL_EMAIL; 

    readonly operationalAlarmsEmail = undefined  // (optional) ex: ops-alarms@example.com
        || process.env.GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL; 

    readonly securityAlarmsEmail = undefined  // (optional) ex: security-alarms@example.com
        || process.env.GRAPL_CDK_SECURITY_ALARMS_EMAIL; 
}
const PARAMS = new DeploymentParameters();

const stackName = PARAMS.deployName;
if (!stackName) {
    throw new Error("Error: Missing Grapl deployment name. Set via bin/grapl-cdk.ts, or environment variable GRAPL_CDK_DEPLOYMENT_NAME.");
}

const app = new cdk.App();

const grapl = new GraplCdkStack(app, 'Grapl', {
    version: PARAMS.graplVersion,
    stackName: stackName,
    watchfulEmail: PARAMS.watchfulEmail,
    operationalAlarmsEmail: PARAMS.operationalAlarmsEmail,
    securityAlarmsEmail: PARAMS.securityAlarmsEmail,
    tags: { 'grapl deployment': stackName},
    description: 'Grapl base deployment',
});

new EngagementUx(app, 'EngagementUX', {
    prefix: grapl.prefix,
    engagement_edge: grapl.engagement_edge,
    graphql_endpoint: grapl.graphql_endpoint,
    model_plugin_deployer: grapl.model_plugin_deployer,
    stackName: stackName + '-EngagementUX',
    description: 'Grapl Engagement UX',
});
