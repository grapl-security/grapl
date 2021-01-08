import * as apigateway from '@aws-cdk/aws-apigateway';
import * as cdk from '@aws-cdk/core';
import * as iam from '@aws-cdk/aws-iam';
import * as lambda from '@aws-cdk/aws-lambda';
import * as s3 from '@aws-cdk/aws-s3';
import { WatchedOperation } from 'cdk-watchful';
import { GraplServiceProps } from '../grapl-cdk-stack';
import { SchemaDb } from '../schemadb';
import { GraplS3Bucket } from '../grapl_s3_bucket';

export interface ModelPluginDeployerProps extends GraplServiceProps {
    modelPluginBucket: s3.IBucket;
    schemaTable: SchemaDb;
    edgeApi: apigateway.RestApi;
}

export class ModelPluginDeployer extends cdk.NestedStack {
    apis: WatchedOperation[];

    constructor(
        parent: cdk.Construct,
        id: string,
        props: ModelPluginDeployerProps
    ) {
        super(parent, id);

        const serviceName = props.prefix + '-ModelPluginDeployer';

        const ux_bucket = GraplS3Bucket.fromBucketName(
            this,
            'uxBucket',
            props.prefix.toLowerCase() + '-engagement-ux-bucket'
        );

        const role = new iam.Role(this, 'ExecutionRole', {
            assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
            roleName: serviceName + '-HandlerRole',
            description: 'Lambda execution role for: ' + serviceName,
            managedPolicies: [
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    'service-role/AWSLambdaBasicExecutionRole'
                ),
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    'service-role/AWSLambdaVPCAccessExecutionRole'
                ),
            ],
        });

        const event_handler = new lambda.Function(this, 'Handler', {
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: `grapl_model_plugin_deployer.app`,
            functionName: serviceName + '-Handler',
            code: lambda.Code.fromAsset(
                `./zips/model-plugin-deployer-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                JWT_SECRET_ID: props.jwtSecret.secretArn,
                USER_AUTH_TABLE: props.userAuthTable.user_auth_table.tableName,
                BUCKET_PREFIX: props.prefix,
                UX_BUCKET_URL: 'https://' + ux_bucket.bucketRegionalDomainName,
                GRAPL_LOG_LEVEL: 'DEBUG',
            },
            timeout: cdk.Duration.seconds(25),
            memorySize: 256,
            description: props.version,
            role,
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
            props.userAuthTable.allowReadFromRole(event_handler.role);
            props.schemaTable.allowReadWriteFromRole(event_handler.role);

            props.modelPluginBucket.grantReadWrite(event_handler.role);
            props.modelPluginBucket.grantDelete(event_handler.role);
        }

        const integration = new apigateway.LambdaIntegration(event_handler);
        props.edgeApi.root.addResource('modelPluginDeployer').addProxy({
            defaultIntegration: integration,
        });
        this.apis = [];
        for (const httpMethod of ['POST', 'OPTIONS', 'GET', 'DELETE']) {
            for (const resourcePath of ['/gitWebhook', '/deploy', '/listModelPlugins', 'deleteModelPlugin', '/{proxy+}']) {
                this.apis.push({ httpMethod, resourcePath });
                this.apis.push({ httpMethod, resourcePath: '/modelPluginDeployer' + resourcePath });
            }
        }
    }
}
