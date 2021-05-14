import * as cdk from "@aws-cdk/core";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as s3 from "@aws-cdk/aws-s3";
import { EventEmitter } from "../event_emitters";
import { RedisCluster } from "../redis";
import { GraplServiceProps } from "../grapl-cdk-stack";
import { ContainerImage } from "@aws-cdk/aws-ecs";
import { FargateService } from "../fargate_service";
import { GraplS3Bucket } from "../grapl_s3_bucket";
import { SRC_DIR, RUST_DOCKERFILE } from "../dockerfile_paths";
import { GraphMutationService } from "./graph_mutation_service";

export interface GraphMergerProps extends GraplServiceProps {
    writesTo: s3.IBucket;
    graphMutationService: GraphMutationService;
}

export class GraphMerger extends cdk.NestedStack {
    readonly bucket: GraplS3Bucket;
    readonly service: FargateService;

    constructor(scope: cdk.Construct, id: string, props: GraphMergerProps) {
        super(scope, id);

        const service_name = "graph-merger";
        const deployment_name = props.deploymentName.toLowerCase();
        const subgraphs_generated = new EventEmitter(
            this,
            deployment_name + "-subgraphs-generated"
        );
        this.bucket = subgraphs_generated.bucket;

        const event_cache = new RedisCluster(
            this,
            "GraphMergerMergedCache",
            props
        );

        event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new FargateService(this, service_name, {
            deploymentName: props.deploymentName,
            environment: {
                REDIS_ENDPOINT: event_cache.address,
                GRAPH_MUTATION_ENDPOINT: props.graphMutationService.endpoint,
                DEPLOYMENT_NAME: deployment_name,
                RUST_LOG: props.logLevels.graphMergerLogLevel,
                SUBGRAPH_MERGED_BUCKET: props.writesTo.bucketName,
                MERGED_CACHE_ADDR: event_cache.cluster.attrRedisEndpointAddress,
                MERGED_CACHE_PORT: event_cache.cluster.attrRedisEndpointPort,
            },
            vpc: props.vpc,
            eventEmitter: subgraphs_generated,
            writesTo: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            serviceImage: ContainerImage.fromAsset(SRC_DIR, {
                target: "graph-merger-deploy",
                buildArgs: {
                    RUST_BUILD: "debug",
                },
                file: RUST_DOCKERFILE,
            }),
            command: ["/graph-merger"],
            metric_forwarder: props.metricForwarder,
        });

        props.graphMutationService.grantAccess(
            this.service.service.service.connections
        );
        props.graphMutationService.grantAccess(
            this.service.retryService.service.connections
        );
    }
}
