import * as cdk from '@aws-cdk/core';
import * as lambda from '@aws-cdk/aws-lambda';
import * as apigateway from '@aws-cdk/aws-apigateway';
import * as s3 from '@aws-cdk/aws-s3';
import { DisplayPropertyDb } from './displaydb';


import { GraplServiceProps } from './grapl-cdk-stack';
import {WatchedOperation} from "cdk-watchful";

export interface GraphQLEndpointProps extends GraplServiceProps {
    ux_bucket: s3.IBucket;
    edgeApi: apigateway.RestApi;
    displayTable: DisplayPropertyDb; 
}

export class GraphQLEndpoint extends cdk.Construct {
    // todo: We should use our own type here imo
    apis: WatchedOperation[];

    constructor(
        parent: cdk.Construct,
        id: string,
        props: GraphQLEndpointProps,
    ) {
        super(parent, id);

        const ux_bucket = props.ux_bucket;

        const serviceName = props.deploymentName + '-GraphQL';

        const event_handler = new lambda.Function(this, 'Handler', {
            runtime: lambda.Runtime.NODEJS_12_X,
            handler: `server.handler`,
            functionName: serviceName + '-Handler',
            code: lambda.Code.fromAsset(
                `./zips/graphql-endpoint-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                JWT_SECRET_ID: props.jwtSecret.secretArn,
                DEPLOYMENT_NAME: props.deploymentName,
                UX_BUCKET_URL: 'https://' + ux_bucket.bucketRegionalDomainName,
                GRAPL_DISPLAY_TABLE: `${props.deploymentName}-grapl_display_table`,
            },
            timeout: cdk.Duration.seconds(30),
            memorySize: 128,
            description: props.version,
        });
        event_handler.currentVersion.addAlias('live');

        props.dgraphSwarmCluster.allowConnectionsFrom(event_handler);

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                event_handler.functionName,
                event_handler
            );
        }

        if (event_handler.role) {
            props.jwtSecret.grantRead(event_handler.role);
            props.displayTable.allowReadWriteFromRole(event_handler.role);

        }
        const integration = new apigateway.LambdaIntegration(event_handler);
        props.edgeApi.root.addResource('graphQlEndpoint').addProxy({
            defaultIntegration: integration,
        });

        this.apis = [];
        for (const httpMethod of ['POST', 'OPTIONS', 'GET', 'DELETE']) {
            for (const resourcePath of ['/graphql', '/{proxy+}']) {
                this.apis.push({httpMethod, resourcePath});
                this.apis.push({httpMethod, resourcePath: '/graphQlEndpoint' + resourcePath});
            }
        }
    }
}
