#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';

import { GraplCdkStack } from '../lib/grapl-cdk-stack';
import { EngagementUx } from '../lib/engagement';
import { DeploymentParameters } from './deployment_parameters';

const app = new cdk.App();

const deploymentParameters = new DeploymentParameters();

const grapl = new GraplCdkStack(app, 'Grapl', {
    version: deploymentParameters.graplVersion,
    logLevels: deploymentParameters.logLevels,
    stackName: deploymentParameters.stackName,
    watchfulEmail: deploymentParameters.watchfulEmail,
    operationalAlarmsEmail: deploymentParameters.operationalAlarmsEmail,
    securityAlarmsEmail: deploymentParameters.securityAlarmsEmail,
    tags: { 'grapl deployment': deploymentParameters.stackName },
    description: 'Grapl base deployment',
    env: { 'region': deploymentParameters.region },
});

new EngagementUx(app, 'EngagementUX', {
    deploymentName: grapl.deploymentName,
    edgeApi: grapl.edgeApiGateway,
    stackName: deploymentParameters.stackName + '-EngagementUX',
    description: 'Grapl Engagement UX',
    env: { 'region': deploymentParameters.region },
});
