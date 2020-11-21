#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';

import { GraplCdkStack } from '../lib/grapl-cdk-stack';
import { EngagementUx } from '../lib/engagement';
import {DeploymentParameters} from './deployment_parameters';

const app = new cdk.App();

const grapl = new GraplCdkStack(app, 'Grapl', {
    version: DeploymentParameters.graplVersion,
    stackName: DeploymentParameters.stackName,
    watchfulEmail: DeploymentParameters.watchfulEmail,
    operationalAlarmsEmail: DeploymentParameters.operationalAlarmsEmail,
    securityAlarmsEmail: DeploymentParameters.securityAlarmsEmail,
    dgraphInstanceType: DeploymentParameters.dgraphInstanceType,
    tags: { 'grapl deployment': DeploymentParameters.stackName},
    description: 'Grapl base deployment',
    env: { 'region': region || process.env.CDK_DEFAULT_REGION }
});

new EngagementUx(app, 'EngagementUX', {
    prefix: grapl.prefix,
    engagement_edge: grapl.engagement_edge,
    graphql_endpoint: grapl.graphql_endpoint,
    model_plugin_deployer: grapl.model_plugin_deployer,
    stackName: DeploymentParameters.stackName + '-EngagementUX',
    description: 'Grapl Engagement UX',
});
