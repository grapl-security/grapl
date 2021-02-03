import * as cdk from '@aws-cdk/core';
import * as s3 from '@aws-cdk/aws-s3';
import * as lambda from '@aws-cdk/aws-lambda';
import * as apigateway from '@aws-cdk/aws-apigateway';

import {GraplServiceProps} from './grapl-cdk-stack';
import {WatchedOperation} from "cdk-watchful";

export interface UxRouterProps extends GraplServiceProps {
    edgeApi: apigateway.RestApi,
}

export class UxRouter extends cdk.NestedStack {
    event_handler: lambda.Function;
    name: string;
    apis: WatchedOperation[];

    constructor(scope: cdk.Construct, id: string, props: UxRouterProps) {
        super(scope, id);

        const ux_bucket = s3.Bucket.fromBucketName(
            this,
            'uxBucket',
            props.prefix.toLowerCase() + '-engagement-ux-bucket'
        );

        const serviceName = props.prefix + '-UxRouter';
        this.name = id + props.prefix;

        this.event_handler = new lambda.Function(this, 'Handler', {
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: `src.grapl_ux_router.app`,
            functionName: serviceName + '-Handler',
            code: lambda.Code.fromAsset(
                `./zips/ux-router-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                UX_BUCKET_NAME: ux_bucket.bucketName,
                GRAPL_LOG_LEVEL: "DEBUG",
            },
            timeout: cdk.Duration.seconds(5),
            memorySize: 128,
            description: props.version,
        });
        this.event_handler.currentVersion.addAlias('live');
        ux_bucket.grantRead(this.event_handler);
        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                this.event_handler.functionName,
                this.event_handler
            );
        }
        if (this.event_handler.role) {
            props.jwtSecret.grantRead(this.event_handler.role);
        }

        const integration = new apigateway.LambdaIntegration(this.event_handler);
        props.edgeApi.root.addProxy({
           defaultIntegration: integration,
        });
        // props.edgeApi.root.addResource("{proxy+}").addProxy({
        //     defaultIntegration: integration,
        // });
        this.apis = [];
        for (const httpMethod of ['POST', 'OPTIONS', 'GET', 'DELETE']) {
            for (const resourcePath of ['/', '/{proxy+}']) {
                this.apis.push({httpMethod, resourcePath});
            }
        }
    }
}
