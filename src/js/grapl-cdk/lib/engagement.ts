import * as cdk from "@aws-cdk/core";
import * as s3 from "@aws-cdk/aws-s3";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as apigateway from "@aws-cdk/aws-apigateway";
import * as sagemaker from "@aws-cdk/aws-sagemaker";
import * as s3deploy from "@aws-cdk/aws-s3-deployment";

import * as aws from "aws-sdk";

import { UserAuthDb } from "./userauthdb";
import { RemovalPolicy } from "@aws-cdk/core";
import { GraphQLEndpoint } from '../lib/graphql';
import { GraplEnvironementProps } from '../lib/grapl-cdk-stack';

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

export class EngagementEdge extends cdk.Stack {
    event_handler: lambda.Function;
    integration: apigateway.LambdaRestApi;
    name: string;
    integrationName: string;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: GraplEnvironementProps,
    ) {
        super(scope, id + 'Stack', { stackName: 'Grapl-EngagementEdge' });

        this.name = id + props.prefix;
        this.integrationName = id + props.prefix + 'Integration';

        this.event_handler = new lambda.Function(
            this, id, {
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: `engagement_edge.app`,
            code: lambda.Code.fromAsset(`./zips/engagement-edge.zip`),
            vpc: props.vpc,
            environment: {
                "EG_ALPHAS": props.engagement_graph.alphaNames.join(","),
                "JWT_SECRET_ID": props.jwt_secret.secretArn,
                "USER_AUTH_TABLE": props.user_auth_table.user_auth_table.tableName,
                "BUCKET_PREFIX": props.prefix,
            },
            timeout: cdk.Duration.seconds(25),
            memorySize: 256,
        }
        );

        if (this.event_handler.role) {
            props.jwt_secret.grantRead(this.event_handler.role);
        }
        props.user_auth_table.allowReadFromRole(this.event_handler);

        this.integration = new apigateway.LambdaRestApi(
            this,
            'Integration',
            {
                handler: this.event_handler,
                restApiName: this.integrationName,
                endpointExportName: "EngagementEndpointApi",
            },
        );
    }
}

export class EngagementNotebook extends cdk.NestedStack {
    securityGroup: ec2.SecurityGroup;
    connections: ec2.Connections;

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        user_auth_db: UserAuthDb,
        vpc: ec2.Vpc,
    ) {
        super(scope, id);

        this.securityGroup = new ec2.SecurityGroup(
            this,
            prefix + '-notebook-security-group',
            { vpc: vpc });

        this.connections = new ec2.Connections({
            securityGroups: [this.securityGroup],
            defaultPort: ec2.Port.allTcp()
        });

        const role = new iam.Role(
            this,
            id + 'notebook-role',
            {
                assumedBy: new iam.ServicePrincipal('sagemaker.amazonaws.com')
            }
        );

        user_auth_db.allowReadWriteFromRole(role);

        const _notebook = new sagemaker.CfnNotebookInstance(
            this,
            id + '-sagemaker-endpoint',
            {
                instanceType: 'ml.t2.medium',
                securityGroupIds: [this.securityGroup.securityGroupId],
                subnetId: vpc.privateSubnets[0].subnetId,
                directInternetAccess: 'Enabled',
                roleArn: role.roleArn
            }
        );

    }
}

export class EngagementUx extends cdk.Stack {
    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        edge: EngagementEdge,
        graphql_endpoint: GraphQLEndpoint,
    ) {
        super(scope, id + 'Stack', { stackName: 'Grapl-EngagementUX' });

        const bucketName = `${prefix}-engagement-ux-bucket`;

        const edgeBucket = new s3.Bucket(this, bucketName, {
            bucketName,
            publicReadAccess: true,
            websiteIndexDocument: 'index.html',
            removalPolicy: RemovalPolicy.DESTROY,
        });

        getEdgeGatewayId(
            [edge.integrationName, graphql_endpoint.integrationName],
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


                new s3deploy.BucketDeployment(this, id + 'Ux', {
                    sources: [s3deploy.Source.asset(packageDir)],
                    destinationBucket: edgeBucket,
                });
            });

    }
}
