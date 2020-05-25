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
        super(parent, name + 'Stack', { stackName: 'Grapl-GraphQLEndpoint' });

        this.name = name + props.prefix
        this.integrationName = name + props.prefix + 'GraphQLIntegration';
        
        this.event_handler = new lambda.Function(
            this, name, {
                runtime: lambda.Runtime.NODEJS_12_X,
                handler: `server.handler`,
                code: lambda.Code.fromAsset(`./zips/graphql-endpoint.zip`),
                vpc: props.vpc,
                environment: {
                    "EG_ALPHAS": props.engagement_graph.alphaNames.join(","),
                    "JWT_SECRET": props.jwt_secret,
                    "BUCKET_PREFIX": props.prefix,
                },
                timeout: cdk.Duration.seconds(25),
                memorySize: 128,
            }
        );

        this.integration = new apigateway.LambdaRestApi(
            this,
            'Integration',
            {
                handler: this.event_handler,
                restApiName: this.integrationName,
                endpointExportName: "GraphQLEndpointApi",
            },
        );
    }
}