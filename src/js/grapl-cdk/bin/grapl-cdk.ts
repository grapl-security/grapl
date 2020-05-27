#!/usr/bin/env node
import 'source-map-support/register';
import * as cdk from '@aws-cdk/core';

import { GraplCdkStack } from '../lib/grapl-cdk-stack';
import { EngagementUx, EngagementEdge } from '../lib/engagement';
import { GraphQLEndpoint } from '../lib/graphql';

const env = require('node-env-file');

env(__dirname + '/../.env');

const app = new cdk.App();
const grapl = new GraplCdkStack(app, 'GraplCdkStack', { stackName: "GraplCDK" });

const engagement_edge = new EngagementEdge(
    app,
    'EngagementEdge',
    grapl.grapl_env
);

const graphql_endpoint = new GraphQLEndpoint(
    app,
    'GraphqlEndpoint',
    grapl.grapl_env
);

const ux = new EngagementUx(
    app,
    'EngagementUX',
    grapl.grapl_env.prefix,
    engagement_edge,
    graphql_endpoint
);
