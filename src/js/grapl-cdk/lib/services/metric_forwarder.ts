import * as cdk from "@aws-cdk/core";
import * as iam from "@aws-cdk/aws-iam";
import { Service } from "../service";
import { GraplServiceProps } from "../grapl-cdk-stack";

export interface MetricForwarderProps extends GraplServiceProps {
    // nothing yet
}

export class MetricForwarder extends cdk.NestedStack {
    readonly service: Service;

    constructor(scope: cdk.Construct, id: string, props: MetricForwarderProps) {
        super(scope, id);

        this.service = new Service(this, id, {
            deploymentName: props.deploymentName,
            environment: {
                GRAPL_LOG_LEVEL: "INFO",
            },
            vpc: props.vpc,
            version: props.version,
            watchful: props.watchful,
            metric_forwarder: undefined, // Otherwise, it'd be recursive!
        });

        const policy = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: ["cloudwatch:PutMetricData"],
            resources: ["*"],
        });

        this.service.event_handler.addToRolePolicy(policy);
        this.service.event_retry_handler.addToRolePolicy(policy);
    }
}
