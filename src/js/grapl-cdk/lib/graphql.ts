import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as apigateway from "@aws-cdk/aws-apigateway";

import { GraplServiceProps } from './grapl-cdk-stack';

export class GraphQLEndpoint extends cdk.Construct {
    integrationName: string;

    constructor(
        parent: cdk.Construct,
        id: string,
        props: GraplServiceProps,
    ) {
        super(parent, id);

        const serviceName = props.prefix + '-GraphQL';
        this.integrationName = serviceName + '-Integration';

        const event_handler = new lambda.Function(
            this, 'Handler', {
                runtime: lambda.Runtime.NODEJS_12_X,
                handler: `server.handler`,
                functionName: serviceName + '-Handler',
                code: lambda.Code.fromAsset(`./zips/graphql-endpoint-${props.version}.zip`),
                vpc: props.vpc,
                environment: {
                    "MG_ALPHAS": props.masterGraph.alphaHostPorts().join(","),
                    "JWT_SECRET_ID": props.jwtSecret.secretArn,
                    "BUCKET_PREFIX": props.prefix,
                },
                timeout: cdk.Duration.seconds(25),
                memorySize: 128,
                description: props.version,
            }
        );
        event_handler.currentVersion.addAlias('live');

        if (event_handler.role) {
            props.jwtSecret.grantRead(event_handler.role);
        }

        const integration = new apigateway.LambdaRestApi(
            this,
            'Integration',
            {
                handler: event_handler,
                restApiName: this.integrationName,
                endpointExportName: serviceName + '-EndpointApi',
            },
        );

        integration.addUsagePlan('graphQLApiUsagePlan', {
            quota: {
                limit: 1_000_000,
                period: apigateway.Period.DAY,
            },
            throttle: {  // per minute
                rateLimit: 5000,
                burstLimit: 10_000,
            }
        });
    }
}
