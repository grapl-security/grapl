import * as cdk from '@aws-cdk/core';
import * as s3 from '@aws-cdk/aws-s3';
import * as sns from '@aws-cdk/aws-sns';
import * as s3n from '@aws-cdk/aws-s3-notifications';

import { RemovalPolicy } from '@aws-cdk/core';
import { BlockPublicAccess, BucketEncryption } from '@aws-cdk/aws-s3';
import { GraplS3Bucket } from './grapl_s3_bucket';

export class EventEmitter {
    readonly bucket: GraplS3Bucket;
    readonly topic: sns.Topic;

    constructor(scope: cdk.Construct, eventName: string) {
        this.bucket = new GraplS3Bucket(scope, eventName + '-bucket', {
            bucketName: eventName + '-bucket',
            removalPolicy: RemovalPolicy.DESTROY,
            encryption: BucketEncryption.KMS_MANAGED,
            blockPublicAccess: BlockPublicAccess.BLOCK_ALL,
        });

        // SNS Topics
        this.topic = new sns.Topic(scope, eventName + '-topic', {
            topicName: eventName + '-topic',
        });

        this.bucket.addEventNotification(
            s3.EventType.OBJECT_CREATED,
            new s3n.SnsDestination(this.topic)
        );

        this.topic.addToResourcePolicy;
    }
}
