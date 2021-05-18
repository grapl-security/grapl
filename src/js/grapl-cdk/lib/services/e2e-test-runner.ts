import * as cdk from "@aws-cdk/core";
import * as iam from "@aws-cdk/aws-iam";
import * as lambda from "@aws-cdk/aws-lambda";

import { GraplServiceProps } from "../grapl-cdk-stack";
import { SchemaDb } from "../schemadb";
import { Provisioner } from "./provisioner";

export interface E2eTestRunnerProps extends GraplServiceProps {
    schemaDb: SchemaDb;
    provisioner: Provisioner;
}

export class E2eTestRunner extends cdk.NestedStack {
    constructor(parent: cdk.Construct, id: string, props: E2eTestRunnerProps) {
        super(parent, id);

        const serviceName = `${props.deploymentName}-E2eTestRunner`;

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
            functionName: `${serviceName}-Handler`,
            code: lambda.Code.fromAsset(
                `./zips/e2e-test-runner-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                GRAPL_LOG_LEVEL: props.logLevels.defaultLogLevel || "INFO",
                DEPLOYMENT_NAME: props.deploymentName,
                GRAPL_TEST_USER_NAME: `${props.deploymentName}-grapl-test-user`,
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
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
        props.provisioner.grantTestUserPasswordRead(event_handler);

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                event_handler.functionName,
                event_handler
            );
        }
    }
}
