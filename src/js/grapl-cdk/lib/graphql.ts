import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as apigateway from "@aws-cdk/aws-apigateway";
import { GraplEnvironementProps } from '../lib/grapl-cdk-stack';

export class GraphQLEndpoint extends cdk.Stack {
    event_handler: lambda.Function;
    integration: apigateway.LambdaRestApi;
    name: string;
    integrationName: string;

    constructor(
        parent: cdk.Construct,
        name: string,
        props: GraplEnvironementProps,
    ) {
        super(parent, name, { stackName: 'Grapl-GraphQLEndpoint' });

        this.name = name + props.prefix
        this.integrationName = name + props.prefix + 'GraphQLIntegration';

        const grapl_version = process.env.GRAPL_VERSION || "latest";

        this.event_handler = new lambda.Function(
            this, 'Handler', {
                runtime: lambda.Runtime.NODEJS_12_X,
                handler: `server.handler`,
                code: lambda.Code.fromAsset(`./zips/graphql-endpoint-${grapl_version}.zip`),
                vpc: props.vpc,
                environment: {
                    "MG_ALPHAS": props.master_graph.alphaHostPorts().join(","),
                    "JWT_SECRET_ID": props.jwt_secret.secretArn,
                    "BUCKET_PREFIX": props.prefix,
                },
                timeout: cdk.Duration.seconds(25),
                memorySize: 128,
                description: grapl_version,
            }
        );
        this.event_handler.currentVersion.addAlias('live');

        if (this.event_handler.role) {
            props.jwt_secret.grantRead(this.event_handler.role);
        }

        this.integration = new apigateway.LambdaRestApi(
            this,
            'Integration',
            {
                handler: this.event_handler,
                restApiName: this.integrationName,
                endpointExportName: "GraphQLEndpointApi",
            },
        );

        this.integration.addUsagePlan('graphQLApiUsagePlan', {
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
