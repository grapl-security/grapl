import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as s3 from '@aws-cdk/aws-s3';
import { EventEmitter } from '../event_emitters';
import { RedisCluster } from '../redis';
import { GraplServiceProps } from '../grapl-cdk-stack';
import { FargateService } from "../fargate_service";
import { ContainerImage } from "@aws-cdk/aws-ecs";
import { SRC_DIR, RUST_DOCKERFILE } from '../dockerfile_paths';

interface OSQueryGraphGeneratorProps extends GraplServiceProps {
    writesTo: s3.IBucket;
}

export class OSQueryGraphGenerator extends cdk.NestedStack {
    readonly service: FargateService;

    constructor(
        parent: cdk.Construct,
        id: string,
        props: OSQueryGraphGeneratorProps
    ) {
        super(parent, id);

        const service_name = "osquery-generator";
        const bucket_prefix = props.prefix.toLowerCase();
        const osquery_log = new EventEmitter(
            this,
            bucket_prefix + '-osquery-log'
        );

        const event_cache = new RedisCluster(this, 'OSQueryEventCache', props);
        event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        this.service = new FargateService(this, service_name, {
            prefix: props.prefix,
            environment: {
                RUST_LOG: props.osquerySubgraphGeneratorLogLevel,
                BUCKET_PREFIX: bucket_prefix,
                EVENT_CACHE_CLUSTER_ADDRESS: event_cache.address,
            },
            vpc: props.vpc,
            eventEmitter: osquery_log,
            writesTo: props.writesTo,
            version: props.version,
            watchful: props.watchful,
            serviceImage: ContainerImage.fromAsset(SRC_DIR, {
                target: "osquery-subgraph-generator-deploy",
                buildArgs: {
                    "CARGO_PROFILE": "debug"
                },
                file: RUST_DOCKERFILE,
            }),
            command: ["/osquery-subgraph-generator"],
            //metric_forwarder: props.metricForwarder,
        });

        for (const conn of this.service.connections()) {
            conn.allowToAnyIpv4(
                ec2.Port.tcp(parseInt(event_cache.cluster.attrRedisEndpointPort))
            );
        }
    }
}
