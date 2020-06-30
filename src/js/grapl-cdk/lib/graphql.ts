import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as apigateway from "@aws-cdk/aws-apigateway";
import * as s3 from "@aws-cdk/aws-s3";

import { GraplServiceProps } from './grapl-cdk-stack';

export class GraphQLEndpoint extends cdk.Construct {
    integrationName: string;

    constructor(
        parent: cdk.Construct,
        id: string,
        props: GraplServiceProps,
        ux_bucket: s3.IBucket,
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
                    "UX_BUCKET_URL": ux_bucket.bucketRegionalDomainName,
                },
                timeout: cdk.Duration.seconds(25),
                memorySize: 128,
                description: props.version,
            }
        );
        event_handler.currentVersion.addAlias('live');

        props.watchful.watchLambdaFunction(event_handler.functionName, event_handler);

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

        props.watchful.watchApiGateway(this.integrationName, integration, {
            serverErrorThreshold: 1, // any 5xx alerts
            cacheGraph: true,
            watchedOperations: [
                {
                    httpMethod: "POST",
                    resourcePath: "/graphql"
                },
            ]
        });

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
