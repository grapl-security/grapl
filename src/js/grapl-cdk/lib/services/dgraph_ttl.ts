import * as cdk from "@aws-cdk/core";
import * as events from "@aws-cdk/aws-events";
import * as iam from "@aws-cdk/aws-iam";
import * as lambda from "@aws-cdk/aws-lambda";
import * as targets from "@aws-cdk/aws-events-targets";
import { GraplServiceProps } from "../grapl-cdk-stack";

export class DGraphTtl extends cdk.NestedStack {
    constructor(parent: cdk.Construct, id: string, props: GraplServiceProps) {
        super(parent, id);

        const serviceName = props.deploymentName + "-DGraphTtl";

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
            functionName: serviceName + "-Handler",
            code: lambda.Code.fromAsset(
                `./zips/dgraph-ttl-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                GRAPL_LOG_LEVEL: props.logLevels.defaultLogLevel,
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                GRAPL_DGRAPH_TTL_S: "2678400", // 60 * 60 * 24 * 31 == 1 month
                GRAPL_TTL_DELETE_BATCH_SIZE: "1000",
            },
            timeout: cdk.Duration.seconds(600),
            memorySize: 128,
            description: props.version,
            role,
        });
        event_handler.currentVersion.addAlias("live");

        props.dgraphSwarmCluster.allowConnectionsFrom(event_handler);

        const target = new targets.LambdaFunction(event_handler);

        const rule = new events.Rule(this, "Rule", {
            schedule: events.Schedule.expression("rate(1 hour)"),
        });
        rule.addTarget(target);

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                event_handler.functionName,
                event_handler
            );
        }
    }
}
