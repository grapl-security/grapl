import * as cdk from '@aws-cdk/core';
import * as s3 from '@aws-cdk/aws-s3';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as lambda from '@aws-cdk/aws-lambda';
import * as iam from '@aws-cdk/aws-iam';
import * as apigateway from '@aws-cdk/aws-apigateway';
import * as sagemaker from '@aws-cdk/aws-sagemaker';
import * as s3deploy from '@aws-cdk/aws-s3-deployment';

import * as aws from 'aws-sdk';

import { GraplServiceProps } from './grapl-cdk-stack';

import * as path from 'path';
import {WatchedOperation} from "cdk-watchful";
import { SchemaDb } from './schemadb';
import { GraplS3Bucket } from './grapl_s3_bucket';


function getEdgeGatewayId(
    edgeApiName: string,
    cb: (edgeApiId: string) => void
) {
    let apigateway = new aws.APIGateway();

    apigateway.getRestApis({}, function (err, data: any) {
        if (err) {
            console.log('Error getting edge gateway ID', err);
        }

        for (const item of data.items) {
            if (item.name === edgeApiName) {
                console.log(`edgeApiId ID ${item.id}`);
                cb(item.id);
                return
            }
        }
    });
}


export interface EngagementEdgeProps extends GraplServiceProps {
    engagement_notebook: EngagementNotebook,
    edgeApi: apigateway.RestApi,
}

export class EngagementEdge extends cdk.NestedStack {
    event_handler: lambda.Function;
    name: string;
    apis: WatchedOperation[];

    constructor(scope: cdk.Construct, id: string, props: EngagementEdgeProps) {
        super(scope, id);

        const ux_bucket = GraplS3Bucket.fromBucketName(
            this,
            'uxBucket',
            props.deploymentName.toLowerCase() + '-engagement-ux-bucket'
        );

        const serviceName = props.deploymentName + '-EngagementEdge';
        this.name = id + props.deploymentName;

        this.event_handler = new lambda.Function(this, 'Handler', {
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: `src.engagement_edge.app`,
            functionName: serviceName + '-Handler',
            code: lambda.Code.fromAsset(
                `./zips/engagement-edge-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                JWT_SECRET_ID: props.jwtSecret.secretArn,
                USER_AUTH_TABLE: props.userAuthTable.user_auth_table.tableName,
                UX_BUCKET_URL: 'https://' + ux_bucket.bucketRegionalDomainName,
                DEPLOYMENT_NAME: props.deploymentName,
            },
            timeout: cdk.Duration.seconds(25),
            memorySize: 256,
            description: props.version,
        });
        this.event_handler.currentVersion.addAlias('live');

        props.dgraphSwarmCluster.allowConnectionsFrom(this.event_handler);

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                this.event_handler.functionName,
                this.event_handler
            );
        }

        // https://github.com/grapl-security/issue-tracker/issues/115
        props.engagement_notebook.allowCreatePresignedUrl(this.event_handler);

        if (this.event_handler.role) {
            props.jwtSecret.grantRead(this.event_handler.role);
        }
        props.userAuthTable.allowReadFromRole(this.event_handler);

        const integration = new apigateway.LambdaIntegration(this.event_handler);
        props.edgeApi.root.addResource('auth').addProxy({
            defaultIntegration: integration,
        });
        this.apis = [];
        for (const httpMethod of ['POST', 'OPTIONS', 'GET', 'DELETE']) {
            for (const resourcePath of ['/login', '/checkLogin', '/{proxy+}']) {
                this.apis.push({httpMethod, resourcePath});
                this.apis.push({httpMethod, resourcePath: '/auth' + resourcePath});
            }
        }
    }
}

export interface EngagementNotebookProps extends GraplServiceProps {
    model_plugins_bucket: s3.IBucket;
    schema_db: SchemaDb;
}

export class EngagementNotebook extends cdk.NestedStack {
    readonly notebookInstance: sagemaker.CfnNotebookInstance;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: EngagementNotebookProps
    ) {
        super(scope, id);

        let serviceName = `${props.deploymentName}-${id}`;
        const securityGroup = new ec2.SecurityGroup(this, 'SecurityGroup', {
            vpc: props.vpc,
        });

        props.dgraphSwarmCluster.allowConnectionsFrom(securityGroup);

        new ec2.Connections({
            securityGroups: [securityGroup],
            defaultPort: ec2.Port.allTcp(),
        });

        const role = new iam.Role(this, 'Role', {
            assumedBy: new iam.ServicePrincipal('sagemaker.amazonaws.com'),
            roleName: serviceName + '-HandlerRole',
            description: 'Notebook role for: ' + serviceName,
        });

        props.userAuthTable.allowReadWriteFromRole(role);
        props.model_plugins_bucket.grantRead(role);
        props.schema_db.allowReadWriteFromRole(role);

        this.notebookInstance = new sagemaker.CfnNotebookInstance(this, 'SageMakerEndpoint', {
            notebookInstanceName: props.deploymentName + '-Notebook',
            instanceType: 'ml.t2.medium',
            securityGroupIds: [securityGroup.securityGroupId],
            subnetId: props.vpc.privateSubnets[0].subnetId,
            directInternetAccess: 'Enabled',
            roleArn: role.roleArn,
        });
    }

    getNotebookArn(): string {
        // there's no better way to get an ARN from a Cfn (low-level Cloudformation) type object.
        if (!this.notebookInstance.notebookInstanceName) {
            throw new Error("gotta have a notebook name");
        }
        return cdk.Arn.format({
            service: "sagemaker",
            resource: "notebook-instance",
            resourceName: this.notebookInstance.notebookInstanceName.toLowerCase(),
        }, this);
    }

    allowCreatePresignedUrl(lambdaFn: lambda.IFunction) {
        const policy = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: ["sagemaker:CreatePresignedNotebookInstanceUrl"],
            resources: [this.getNotebookArn()],
        });

        lambdaFn.addToRolePolicy(policy);
    }
}

interface EngagementUxProps extends cdk.StackProps {
    deploymentName: string;
    edgeApi: apigateway.RestApi;
}

const packageDir = path.join(__dirname, '../edge_ux_post_replace/');
export class EngagementUx extends cdk.Stack {
    constructor(scope: cdk.Construct, id: string, props: EngagementUxProps) {
        super(scope, id, props);

        const edgeBucket = GraplS3Bucket.fromBucketName(
            this,
            'uxBucket',
            props.deploymentName.toLowerCase() + '-engagement-ux-bucket'
        );

        new s3deploy.BucketDeployment(this, 'UxDeployment', {
            sources: [s3deploy.Source.asset(packageDir)],
            destinationBucket: edgeBucket,
        });
    }
}
