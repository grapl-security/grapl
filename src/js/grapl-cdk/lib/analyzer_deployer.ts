import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as s3 from "@aws-cdk/aws-s3";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as ecs from "@aws-cdk/aws-ecs";
import * as iam from "@aws-cdk/aws-iam";

export class AnalyzerDeployer extends cdk.NestedStack {
    event_handler: lambda.Function;
    name: string;

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        grapl_version: string,
        analyzer_request_bucket: s3.IBucket,
        analyzer_matched_bucket: s3.IBucket,
        model_plugins_bucket: s3.IBucket,
        vpc: ec2.Vpc,
    ) {
        super(scope, id);

        const cluster = new ecs.Cluster(this, id + '-FargateCluster', {
            vpc: vpc
        });

        const task = new ecs.FargateTaskDefinition(
            this,
            id,
            {
                cpu: 256,
                memoryLimitMiB: 512,
            }
        );

        task.addContainer(id + 'Container', {
            image: ecs.ContainerImage.fromRegistry("grapl/grapl-analyzer-deployer"),
            environment: {
                CUSTOMER_PREFIX: prefix,
                GRAPL_VERSION: grapl_version,
                VPC_ID: vpc.vpcId,
                ANALYZER_REQUEST_BUCKET: analyzer_request_bucket.bucketArn,
                ANALYZER_MATCHED_BUCKET_ARN: analyzer_matched_bucket.bucketArn,
                MODEL_PLUGIN_BUCKET_ARN: model_plugins_bucket.bucketArn,
                MG_ALPHAS: master_graph.alphaHostPorts().join(",")
            }
        });

        new ecs.FargateService(this, id + 'Service', {
            cluster,  // Required
            taskDefinition: task,
        });

        // Give it permissions to create and modify the Analyzer stack
        // This should be safe because this only grants access to one specific stack, which incorporates
        // the customer name
        const cfPolicy = new iam.PolicyStatement();

        cfPolicy.addActions('cloudformation:*');
        // TODO: Limit region and account as well
        cfPolicy.addResources(`arn:aws:cloudformation:::stack/${prefix}-Analyzers-*-pack/*`);
        task.addToTaskRolePolicy(cfPolicy);
    }
}

export default AnalyzerDeployer;
