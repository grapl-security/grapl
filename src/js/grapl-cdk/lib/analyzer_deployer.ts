import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as apigateway from "@aws-cdk/aws-apigateway";
import * as s3 from "@aws-cdk/aws-s3";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as ecs from "@aws-cdk/aws-ecs";
import * as servicediscovery from "@aws-cdk/aws-servicediscovery";
import * as secretsmanager from '@aws-cdk/aws-secretsmanager';
import {DGraphEcs} from "./dgraph";

class AnalyzerDeployer extends cdk.NestedStack {
    event_handler: lambda.Function;
    integration: apigateway.LambdaRestApi;
    name: string;
    integrationName: string;

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        grapl_version: string,
        merged_graph_bucket: s3.IBucket,
        analyzer_matched_bucket: s3.IBucket,
        model_plugins_bucket: s3.IBucket,
        master_graph: DGraphEcs,
        vpc: ec2.Vpc,
    ) {
        super(scope, id);

        const cluster = new ecs.Cluster(this, id+'-FargateCluster', {
            vpc: vpc
        });

        const zeroTask = new ecs.FargateTaskDefinition(
            this,
            id,
            {
                cpu: 256,
                memoryLimitMiB: 512,
            }
        );

        zeroTask.addContainer(id + 'Container', {
            image: ecs.ContainerImage.fromRegistry("grapl/grapl-analyzer-deployer"),
            environment: {
                CUSTOMER_PREFIX: prefix,
                GRAPL_VERSION: grapl_version,
                VPC_ID: vpc.vpcId,
                MERGED_GRAPH_BUCKET_ARN: merged_graph_bucket.bucketArn,
                ANALYZER_MATCHED_BUCKET_ARN: analyzer_matched_bucket.bucketArn,
                MODEL_PLUGIN_BUCKET_ARN: model_plugins_bucket.bucketArn,
                MG_ALPHAS: master_graph.alphaNames.join(",")
            }
        });

        new ecs.FargateService(this, id+'Service', {
            cluster,  // Required
            taskDefinition: zeroTask,
        });
    }
}

export default AnalyzerDeployer;