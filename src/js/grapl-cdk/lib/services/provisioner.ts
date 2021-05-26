import * as cdk from "@aws-cdk/core";
import * as iam from "@aws-cdk/aws-iam";
import * as lambda from "@aws-cdk/aws-lambda";
import * as secretsmanager from "@aws-cdk/aws-secretsmanager";

import { GraplServiceProps } from "../grapl-cdk-stack";
import { SchemaDb } from "../schemadb";

export interface ProvisionerProps extends GraplServiceProps {
    schemaDb: SchemaDb;
}

export class Provisioner extends cdk.NestedStack {
    private secret: secretsmanager.Secret;

    constructor(parent: cdk.Construct, id: string, props: ProvisionerProps) {
        super(parent, id);

        const serviceName = `${props.deploymentName}-Provisioner`;

        const role = new iam.Role(this, "ExecutionRole", {
            assumedBy: new iam.ServicePrincipal("lambda.amazonaws.com"),
            roleName: serviceName + "-HandlerRole",
            description: "Lambda execution role for: " + serviceName,
            managedPolicies: [
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    "service-role/AWSLambdaBasicExecutionRole"
                ),
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    "service-role/AWSLambdaVPCAccessExecutionRole"
                ),
            ],
        });

        const event_handler = new lambda.Function(this, "Handler", {
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: "lambdex_handler.handler",
            functionName: "provisioner",
            code: lambda.Code.fromAsset(
                `./zips/provisioner-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                GRAPL_LOG_LEVEL: props.logLevels.defaultLogLevel || "INFO",
                DEPLOYMENT_NAME: props.deploymentName,
                GRAPL_TEST_USER_NAME: `${props.deploymentName}-grapl-test-user`,
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                DGRAPH_HOST: props.dgraphSwarmCluster.alphaHost(),
                DGRAPH_ALPHA_HTTP_EXTERNAL_PUBLIC_PORT: `${props.dgraphSwarmCluster.alphaPort()}`,
            },
            timeout: cdk.Duration.seconds(600),
            memorySize: 128,
            description: props.version,
            role,
        });
        event_handler.currentVersion.addAlias("live");

        props.dgraphSwarmCluster.allowConnectionsFrom(event_handler);
        props.userAuthTable.allowReadWriteFromRole(event_handler);
        props.schemaDb.allowReadWriteFromRole(event_handler);

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                event_handler.functionName,
                event_handler
            );
        }

        this.secret = new secretsmanager.Secret(this, "TestUserPassword", {
            secretName: `${props.deploymentName}-TestUserPassword`,
            generateSecretString: {
                passwordLength: 48,
            },
        });
        this.secret.grantRead(role);
    }

    public grantTestUserPasswordRead(grantee: iam.IGrantable): void {
        this.secret.grantRead(grantee);
    }
}
