import * as cdk from "@aws-cdk/core";
import * as s3 from "@aws-cdk/aws-s3";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as apigateway from "@aws-cdk/aws-apigateway";
import * as sagemaker from "@aws-cdk/aws-sagemaker";
import * as s3deploy from "@aws-cdk/aws-s3-deployment";

import * as aws from "aws-sdk";

import { GraplServiceProps } from './grapl-cdk-stack';
import { RemovalPolicy } from "@aws-cdk/core";
import { GraphQLEndpoint } from './graphql';

import * as fs from 'fs';
import * as path from 'path';
import * as dir from 'node-dir';

function getEdgeGatewayId(
    [loginName, graphqlName]: [string, string],
    cb: any
) {
    let apigateway = new aws.APIGateway();

    apigateway.getRestApis({}, function(err, data: any) {
        let edgeId = undefined;
        let graphId = undefined;

        if (err) {
            console.log('Error getting edge gateway ID', err);
        }

        for (const item of data.items) {
            if (item.name === loginName) {
                console.log(`login restApi ID ${item.id}`);
                edgeId = item.id;
                continue
            }
            if (item.name === graphqlName) {
                console.log(`graphql restApi ID ${item.id}`);
                graphId = item.id;
                continue
            }

            if (edgeId && graphId) {
                break
            }
        }

        if (edgeId && graphId) {
            cb(edgeId, graphId);
        } else {
            console.warn(false, 'Could not find any integrations. Ensure you have deployed engagement edge.')
        }
    });
};

function replaceInFile(
    toModify: string,
    replaceMap: Map<string, string>,
    outputFile: string) {
    return fs.readFile(toModify, {encoding: 'utf8'}, (err, data) => {
        if (err) {
            return console.log(err);
        }

        let replaced = data;
        for (const [toReplace, replaceWith] of replaceMap.entries()) {
            replaced = replaced
            .split(toReplace)
            .join(replaceWith);
        }

        if (outputFile) {
            fs.writeFile(outputFile, replaced, {encoding: 'utf8'}, (err: any) => {
                if (err) return console.log(err);
            });
        } else {
            fs.writeFile(toModify, replaced, {encoding: 'utf8'}, (err: any) => {
                if (err) return console.log(err);
            });
        }

    });
};

export class EngagementEdge extends cdk.NestedStack {
    event_handler: lambda.Function;
    integration: apigateway.LambdaRestApi;
    name: string;
    integrationName: string;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: GraplServiceProps,
    ) {
        super(scope, id);

        const serviceName = props.prefix + '-EngagementEdge';
        this.name = id + props.prefix;
        this.integrationName = id + props.prefix + 'Integration';

        this.event_handler = new lambda.Function(
            this, 'Handler', {
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: `engagement_edge.app`,
            functionName: serviceName + '-Handler',
            code: lambda.Code.fromAsset(`./zips/engagement-edge-${props.version}.zip`),
            vpc: props.vpc,
            environment: {
                "MG_ALPHAS": props.masterGraph.alphaHostPorts().join(","),
                "JWT_SECRET_ID": props.jwtSecret.secretArn,
                "USER_AUTH_TABLE": props.userAuthTable.user_auth_table.tableName,
                "BUCKET_PREFIX": props.prefix,
            },
            timeout: cdk.Duration.seconds(25),
            memorySize: 256,
            description: props.version,
        });
        this.event_handler.currentVersion.addAlias('live');

        if (this.event_handler.role) {
            props.jwtSecret.grantRead(this.event_handler.role);
        }
        props.userAuthTable.allowReadFromRole(this.event_handler);

        this.integration = new apigateway.LambdaRestApi(
            this,
            'Integration',
            {
                handler: this.event_handler,
                restApiName: this.integrationName,
                endpointExportName: serviceName + '-EndpointApi',
            },
        );

        this.integration.addUsagePlan('loginApiUsagePlan', {
            quota: {
                limit: 100_000,
                period: apigateway.Period.DAY,
            },
            throttle: {  // per minute
                rateLimit: 500,
                burstLimit: 500,
            }
        });
    }
}

export class EngagementNotebook extends cdk.NestedStack {

    constructor(
        scope: cdk.Construct,
        id: string,
        props: GraplServiceProps,
    ) {
        super(scope, id);

        const securityGroup = new ec2.SecurityGroup(this, 'SecurityGroup', {
            vpc: props.vpc
        });

        new ec2.Connections({
            securityGroups: [securityGroup],
            defaultPort: ec2.Port.allTcp()
        });

        const role = new iam.Role(this, 'Role',{
            assumedBy: new iam.ServicePrincipal('sagemaker.amazonaws.com')
        });

        props.userAuthTable.allowReadWriteFromRole(role);

        new sagemaker.CfnNotebookInstance(
            this,
            'SageMakerEndpoint',
            {
                notebookInstanceName: props.prefix + '-Notebook',
                instanceType: 'ml.t2.medium',
                securityGroupIds: [securityGroup.securityGroupId],
                subnetId: props.vpc.privateSubnets[0].subnetId,
                directInternetAccess: 'Enabled',
                roleArn: role.roleArn
            }
        );
    }
}

interface EngagementUxProps extends cdk.StackProps {
    prefix: string,
    engagement_edge: EngagementEdge,
    graphql_endpoint: GraphQLEndpoint,
}

export class EngagementUx extends cdk.Stack {
    constructor(
        scope: cdk.Construct,
        id: string,
        props: EngagementUxProps
    ) {
        super(scope, id, props);

        const edgeBucket = new s3.Bucket(this, 'EdgeBucket', {
            bucketName: props.prefix.toLowerCase() + '-engagement-ux-bucket',
            publicReadAccess: true,
            websiteIndexDocument: 'index.html',
            removalPolicy: RemovalPolicy.DESTROY
        });

        getEdgeGatewayId(
            [props.engagement_edge.integrationName, props.graphql_endpoint.integrationName],
            (loginGatewayId: string, graphQLGatewayId: string) => {
                const srcDir = path.join(__dirname, "../edge_ux/");
                const packageDir = path.join(__dirname, "../edge_ux_package/");

                if (!fs.existsSync(packageDir)) {
                    fs.mkdirSync(packageDir);
                }

                const loginUrl = `https://${loginGatewayId}.execute-api.${aws.config.region}.amazonaws.com/prod/`;
                const graphQLUrl = `https://${graphQLGatewayId}.execute-api.${aws.config.region}.amazonaws.com/prod/`;

                const replaceMap = new Map();
                replaceMap.set(`http://"+window.location.hostname+":8900/`, loginUrl);
                replaceMap.set(`http://"+window.location.hostname+":5000/`, graphQLUrl);

                dir.readFiles(srcDir,
                    function(err: any, content: any, filename: string, next: any) {
                        if (err) throw err;

                        const targetDir = path.dirname(filename).replace("edge_ux", "edge_ux_package");

                        if (!fs.existsSync(targetDir)) {
                            fs.mkdirSync(targetDir, { recursive: true });
                        }

                        const newPath = filename.replace("edge_ux", "edge_ux_package");

                        replaceInFile(
                            filename,
                            replaceMap,
                            newPath
                        )
                        next()
                    },
                    function(err: any, files: any) {
                        if (err) throw err;
                    });


                new s3deploy.BucketDeployment(this, 'UxDeployment', {
                    sources: [s3deploy.Source.asset(packageDir)],
                    destinationBucket: edgeBucket,
                });
            });

    }
}
