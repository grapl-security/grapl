import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as lambda from '@aws-cdk/aws-lambda';
import * as sns from '@aws-cdk/aws-sns';
import * as s3 from '@aws-cdk/aws-s3';
import {Service} from '../service';
import {EventEmitter} from '../event_emitters';
import {GraplServiceProps} from '../grapl-cdk-stack';

export interface EngagementCreatorProps extends GraplServiceProps {
    publishesTo: sns.ITopic;
}

export class EngagementCreator extends cdk.NestedStack {
    readonly bucket: s3.Bucket;
    readonly service: Service;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: EngagementCreatorProps
    ) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const analyzer_matched_sugraphs = new EventEmitter(
            this,
            bucket_prefix + '-analyzer-matched-subgraphs'
        );
        this.bucket = analyzer_matched_sugraphs.bucket;

        this.service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                GRAPL_LOG_LEVEL: props.analyzerExecutorLogLevel,
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
            },
            vpc: props.vpc,
            reads_from: analyzer_matched_sugraphs.bucket,
            subscribes_to: analyzer_matched_sugraphs.topic,
            opt: {
                runtime: lambda.Runtime.PYTHON_3_7,
            },
            version: props.version,
            watchful: props.watchful,
            metric_forwarder: props.metricForwarder,
        });

        props.dgraphSwarmCluster.allowConnectionsFrom(this.service.event_handler);

        this.service.publishesToTopic(props.publishesTo);

        this.service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.allTcp(),
            'Allow outbound to S3'
        );
        this.service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.allTcp(),
            'Allow outbound to S3'
        );
    }
}
