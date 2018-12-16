import { SqsEventSource } from '@aws-cdk/aws-lambda-event-sources';
import cdk = require('@aws-cdk/cdk');
import s3 = require('@aws-cdk/aws-s3');
import sns = require('@aws-cdk/aws-sns');
import sqs = require('@aws-cdk/aws-sqs');
import ec2 = require('@aws-cdk/aws-ec2');
import rds = require('@aws-cdk/aws-rds');
import lambda = require('@aws-cdk/aws-lambda');
import iam = require('@aws-cdk/aws-iam');
import {SubnetType, VpcNetwork, VpcNetworkRef} from "@aws-cdk/aws-ec2";
import {Token} from "@aws-cdk/cdk";
import {Bucket, BucketRef} from "@aws-cdk/aws-s3";
import {Topic, TopicRef} from "@aws-cdk/aws-sns";
import {DatabaseCluster} from "@aws-cdk/aws-rds";
import {QueueRef} from "@aws-cdk/aws-sqs";


var env = require('node-env-file');

function get_history_db(stack: cdk.Stack, vpc: ec2.VpcNetworkRef, username: Token, password: Token): DatabaseCluster {
    return new rds.DatabaseCluster(stack, 'HistoryDb', {
        defaultDatabaseName: 'historydb',
        masterUser: {
            username: username.toString(),
            password: password.toString(),
        },
        engine: rds.DatabaseClusterEngine.Aurora,
        instanceProps: {
            instanceType: new ec2.InstanceTypePair(
                ec2.InstanceClass.Burstable2,
                ec2.InstanceSize.Medium
            ),
            vpc: vpc,
            vpcPlacement: {
                subnetsToUse: SubnetType.Private
            }
        }
    });

}

class Queues {
    queue: sqs.Queue;
    retry_queue: sqs.Queue;
    dead_letter_queue: sqs.Queue;

    constructor(stack: cdk.Stack, queue_name: string) {
        this.dead_letter_queue = new sqs.Queue(stack, queue_name + '-dead-letter');

        this.retry_queue = new sqs.Queue(stack, queue_name + '-retry', {
            deadLetterQueue: {queue: this.dead_letter_queue, maxReceiveCount: 10},
            visibilityTimeoutSec: 65
        });

        this.queue = new sqs.Queue(stack, queue_name, {
            deadLetterQueue: {queue: this.retry_queue, maxReceiveCount: 5},
            visibilityTimeoutSec: 50
        });

    }
}

class Service {
    event_handler: lambda.Function;
    event_retry_handler: lambda.Function;
    queues: Queues;

    constructor(stack: cdk.Stack, name: string, environment?: any, vpc?: VpcNetworkRef, retry_code_name?: string) {
        const queues = new Queues(stack, name + '-queue');

        let event_handler = new lambda.Function(
            stack, name, {
                runtime: lambda.Runtime.Go1x,
                handler: name,
                code: lambda.Code.asset(`./${name}.zip`),
                vpc: vpc,
                environment: environment,
                timeout: 45,
                memorySize: 256,
            }
        );

        if (!retry_code_name) {
            retry_code_name = name
        }

        let event_retry_handler = new lambda.Function(
            stack, name + '-retry-handler', {
                runtime: lambda.Runtime.Go1x,
                handler: retry_code_name,
                code: lambda.Code.asset(`./${retry_code_name}.zip`),
                vpc: vpc,
                environment: environment,
                timeout: 60,
                memorySize: 512,
            }
        );

        event_handler.addEventSource(new SqsEventSource(queues.queue));
        event_retry_handler.addEventSource(new SqsEventSource(queues.retry_queue));

        this.queues = queues;
        this.event_handler = event_handler;
        this.event_retry_handler = event_retry_handler;
    }

    readsFrom(bucket: BucketRef) {
        this.event_handler.addToRolePolicy(new iam.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addResource(bucket.bucketArn));

        this.event_retry_handler.addToRolePolicy(new iam.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addResource(bucket.bucketArn));

        bucket.grantRead(this.event_handler.role);
        bucket.grantRead(this.event_retry_handler.role);
    }

    publishesTo(publishes_to: TopicRef | BucketRef) {
        if (publishes_to instanceof TopicRef) {
            this.event_handler.addToRolePolicy(new iam.PolicyStatement()
                .addAction('sns:CreateTopic')
                .addResource(publishes_to.topicArn));

            this.event_retry_handler.addToRolePolicy(new iam.PolicyStatement()
                .addAction('sns:CreateTopic')
                .addResource(publishes_to.topicArn));

            publishes_to.grantPublish(this.event_handler.role);
            publishes_to.grantPublish(this.event_retry_handler.role);

        } else {
            publishes_to.grantWrite(this.event_handler.role);
            publishes_to.grantWrite(this.event_retry_handler.role);
        }
    }
}


class SessionIdentityCache extends cdk.Stack {
    constructor(parent: cdk.App, vpc_props: ec2.VpcNetworkRefProps) {
        super(parent, 'session-identity-cache-stack');

        // const zone = new route53.PrivateHostedZone(this, 'HostedZone', {
        //     zoneName: 'sessionid.cache',
        //     vpc_props
        // });


    }

}

class EventEmitters extends cdk.Stack {
    raw_logs_bucket: s3.BucketRefProps;
    sysmon_logs_bucket: s3.BucketRefProps;
    identity_mappings_bucket: s3.BucketRefProps;
    unid_subgraphs_generated_bucket: s3.BucketRefProps;
    subgraphs_generated_bucket: s3.BucketRefProps;

    incident_topic: sns.TopicRefProps;
    identity_mappings_topic: sns.TopicRefProps;
    raw_logs_topic: sns.TopicRefProps;
    sysmon_logs_topic: sns.TopicRefProps;
    unid_subgraphs_generated_topic: sns.TopicRefProps;
    subgraphs_generated_topic: sns.TopicRefProps;
    subgraph_merged_topic: sns.TopicRefProps;

    constructor(parent: cdk.App, id: string) {
        super(parent, id + '-stack');
        let raw_logs_bucket = new s3.Bucket(
            this,
            id + '-raw-log-bucket',
            {
                bucketName: process.env.BUCKET_PREFIX + "-raw-log-bucket"
            });

        let sysmon_logs_bucket = new s3.Bucket(
            this,
            id + '-sysmon-log-bucket',
            {
                bucketName: process.env.BUCKET_PREFIX + "-sysmon-log-bucket"
            });

        let identity_mappings_bucket = new s3.Bucket(
            this,
            id + '-identity-mappings-bucket',
            {
                bucketName: process.env.BUCKET_PREFIX + "-identity-mappings-bucket"
            });

        let unid_subgraphs_generated_bucket = new s3.Bucket(
            this,
            id + '-unid-subgraphs-generated-bucket',
            {
                bucketName: process.env.BUCKET_PREFIX + "-unid-subgraphs-generated-bucket"
            }
        );
        let subgraphs_generated_bucket =
            new s3.Bucket(this, id + '-subgraphs-generated-bucket', {
                bucketName: process.env.BUCKET_PREFIX + "-subgraphs-generated-bucket"
            });

        // SNS Topics
        let incident_topic =
            new sns.Topic(this, id + '-incident-topic', {
                topicName: 'incident-topic'
            });
        let raw_logs_topic =
            new sns.Topic(this, id +  '-raw-log-topic', {
                topicName: 'raw-log-topic'
            });
        let sysmon_logs_topic =
            new sns.Topic(this, id +  '-sysmon-log-topic', {
                topicName: 'sysmon-log-topic'
            });
        let identity_mappings_topic =
            new sns.Topic(this, id +  '-identity-mappings-topic', {
                topicName: 'identity-mappings-topic'
            });
        let unid_subgraphs_generated_topic =
            new sns.Topic(this, id +  '-unid-subgraphs-generated-topic', {
                topicName: 'unid-subgraphs-generated-topic'
            });
        let subgraphs_generated_topic =
            new sns.Topic(this, id + '-subgraphs-generated-topic', {
                topicName: 'subgraphs-generated-topic'
            });
        let subgraph_merged_topic =
            new sns.Topic(this, id + '-subgraphs-merged-topic', {
                topicName: 'subgraphs-merged-topic'
            });

        // S3 -> SNS Events

        raw_logs_bucket
            .onEvent(s3.EventType.ObjectCreated, raw_logs_topic);
        sysmon_logs_bucket
            .onEvent(s3.EventType.ObjectCreated, sysmon_logs_topic);
        identity_mappings_bucket
            .onEvent(s3.EventType.ObjectCreated, identity_mappings_topic);
        unid_subgraphs_generated_bucket
            .onEvent(s3.EventType.ObjectCreated, unid_subgraphs_generated_topic);
        subgraphs_generated_bucket
            .onEvent(s3.EventType.ObjectCreated, subgraphs_generated_topic);

        this.raw_logs_bucket = raw_logs_bucket.export();
        this.sysmon_logs_bucket = sysmon_logs_bucket.export();
        this.identity_mappings_bucket = identity_mappings_bucket.export();
        this.unid_subgraphs_generated_bucket = unid_subgraphs_generated_bucket.export();
        this.subgraphs_generated_bucket = subgraphs_generated_bucket.export();

        this.incident_topic = incident_topic.export();
        this.raw_logs_topic = raw_logs_topic.export();
        this.sysmon_logs_topic = sysmon_logs_topic.export();
        this.identity_mappings_topic = identity_mappings_topic.export();
        this.unid_subgraphs_generated_topic = unid_subgraphs_generated_topic.export();
        this.subgraphs_generated_topic = subgraphs_generated_topic.export();
        this.subgraph_merged_topic = subgraph_merged_topic.export();
    }
}

class SysmonSubgraphGenerator extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketRefProps,
                event_producer_props: sns.TopicRefProps,
                writes_to_props: s3.BucketRefProps,
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const event_producer = Topic.import(
            this,
            'event_producer',
            event_producer_props
        );

        const writes_to = Bucket.import(
            this,
            'writes_to',
            writes_to_props
        );

        const environment = {
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX
        };

        const service = new Service(this, 'sysmon-subgraph-generator', environment);

        service.readsFrom(reads_from);
        event_producer.subscribeQueue(service.queues.queue);
        service.publishesTo(writes_to);
    }
}


class GenericSubgraphGenerator extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketRefProps,
                event_producer_props: sns.TopicRefProps,
                writes_to_props: s3.BucketRefProps,
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const event_producer = Topic.import(
            this,
            'event_producer',
            event_producer_props
        );

        const writes_to = Bucket.import(
            this,
            'writes_to',
            writes_to_props
        );

        const environment = {
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX
        };

        const service = new Service(this, 'generic-subgraph-generator', environment);

        service.readsFrom(reads_from);
        event_producer.subscribeQueue(service.queues.queue);
        service.publishesTo(writes_to);
    }
}


class NodeIdentityMapper extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketRefProps,
                event_producer_props: sns.TopicRefProps,
                history_db_props: rds.DatabaseClusterRefProps,
                vpc_props: ec2.VpcNetworkRefProps
    ) {
        super(parent, id + '-stack');
        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const history_db = rds.DatabaseCluster.import(
            this,
            'history-db',
            history_db_props
        );

        const event_producer = Topic.import(
            this,
            'event_producer',
            event_producer_props
        );

        const vpc = ec2.VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );

        let environment = {
            "HISTORY_DB_USERNAME": process.env.HISTORY_DB_USERNAME,
            "HISTORY_DB_PASSWORD": process.env.HISTORY_DB_PASSWORD,
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX
        };

        let service = new Service(this, 'node-identity-mapper', environment, vpc);


        service.readsFrom(reads_from);

        history_db.connections.allowDefaultPortFrom(
            service.event_handler,
            'node-identity-mapper-history-db'
        );

        history_db.connections.allowDefaultPortFrom(
            service.event_retry_handler,
            'node-identity-mapper-retry-handler-history-db'
        );
        event_producer.subscribeQueue(service.queues.queue);

        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
    }
}


class NodeIdentifier extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketRefProps,
                event_producer_props: sns.TopicRefProps,
                writes_to_props: s3.BucketRefProps,
                history_db_props: rds.DatabaseClusterRefProps,
                vpc_props: ec2.VpcNetworkRefProps
    ) {
        super(parent, id + '-stack');
        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const history_db = rds.DatabaseCluster.import(
            this,
            'history-db',
            history_db_props
        );

        const event_producer = Topic.import(
            this,
            'event_producer',
            event_producer_props
        );

        const writes_to = Bucket.import(
            this,
            'writes_to',
            writes_to_props
        );

        const vpc = ec2.VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );
            
        const environment = {
            "HISTORY_DB_USERNAME": process.env.HISTORY_DB_USERNAME,
            "HISTORY_DB_PASSWORD": process.env.HISTORY_DB_PASSWORD,
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "IDENTITY_CACHE_PEPPER": process.env.IDENTITY_CACHE_PEPPER,
        };

        const service = new Service(this, 'node-identifier', environment, vpc, 'node-identifier-retry-handler');
        service.readsFrom(reads_from);

        history_db.connections.allowDefaultPortFrom(
            service.event_handler,
            'node-identifier-history-db'
        );

        history_db.connections.allowDefaultPortFrom(
            service.event_retry_handler,
            'node-identifier-retry-handler-history-db'
        );

        service.publishesTo(writes_to);
        event_producer.subscribeQueue(service.queues.queue);
        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');

    }
}

class GraphMerger extends cdk.Stack {

    constructor(parent: cdk.App,
                id: string,
                reads_from_props: s3.BucketRefProps,
                event_producer_props: sns.TopicRefProps,
                publishes_to_props: sns.TopicRefProps,
                vpc_props: ec2.VpcNetworkRefProps
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const event_producer = Topic.import(
            this,
            'event_producer',
            event_producer_props
        );

        const publishes_to = sns.Topic.import(
            this,
            'publishes_to',
            publishes_to_props
        );

        const vpc = ec2.VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );

        const environment = {
            "GRAPH_MERGER": process.env.GRAPH_MERGER_READ_BUCKET,
            "GRAPH_MERGER_WRITE_TOPIC": process.env.GRAPH_MERGER_WRITE_TOPIC,
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX
        };

        const service = new Service(this, 'graph-merger', environment, vpc);

        service.readsFrom(reads_from);
        service.publishesTo(publishes_to);

        event_producer.subscribeQueue(service.queues.queue);

        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');

    }
}


class EngagementCreationService extends cdk.Stack {

    constructor(parent: cdk.App,
                id: string,
                event_producer_props: sns.TopicRefProps,
                vpc_props: ec2.VpcNetworkRefProps
    ) {
        super(parent, id + '-stack');

        const event_producer = Topic.import(
            this,
            'event_producer',
            event_producer_props
        );

        const vpc = ec2.VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );

        let engagement_creation_service = new lambda.Function(
            this, 'engagement-creation-service', {
                runtime: lambda.Runtime.Go1x,
                handler: 'engagement-creation-service',
                code: lambda.Code.asset('./engagement-creation-service.zip'),
                vpc: vpc
            }
        );

        let engagement_creation_service_queue =
            new sqs.Queue(this, 'engagement-creation-service-queue');

        // fn.addEventSource(new SqsEventSource(engagement_creation_service_queue));
        // subscribe_lambda_to_queue(
        //     this,
        //     'engagementCreator',
        //     engagement_creation_service,
        //     engagement_creation_service_queue
        // );
        //
        // event_producer.subscribeQueue(engagement_creation_service_queue);
    }
}

class Networks extends cdk.Stack {
    grapl_vpc: ec2.VpcNetworkRefProps;

    constructor(parent: cdk.App, id: string,) {
        super(parent, id + '-stack');

        let grapl_vpc = new ec2.VpcNetwork(this, 'GraplVPC', {
            natGateways: 2
        });

        this.grapl_vpc = grapl_vpc.export();

    }
}

class HistoryDb extends cdk.Stack {

    db: rds.DatabaseClusterRefProps;

    constructor(parent: cdk.App,
                id: string,
                grapl_vpc_props: ec2.VpcNetworkRefProps,
                username: cdk.Token,
                password: cdk.Token
    ) {
        super(parent, id + '-stack');

        const grapl_vpc = ec2.VpcNetwork.import(
            this,
            'vpc',
            grapl_vpc_props
        );

        this.db = get_history_db(this, grapl_vpc, username, password).export();
    }
}

class Grapl extends cdk.App {
    constructor() {
        super();

        const env_file = env(__dirname + '/.env');

        const network = new Networks(this, 'vpcs');

        const history_db = new HistoryDb(
            this,
            'history-db',
            network.grapl_vpc,
            new cdk.Token(process.env.HISTORY_DB_USERNAME),
            new cdk.Token(process.env.HISTORY_DB_PASSWORD)
        );

        let event_emitters = new EventEmitters(this, 'event-emitters');
        new GenericSubgraphGenerator(
            this,
            'generic-subgraph-generator',
            event_emitters.raw_logs_bucket,
            event_emitters.raw_logs_topic,
            event_emitters.unid_subgraphs_generated_bucket
        );

        new SysmonSubgraphGenerator(
            this,
            'sysmon-subgraph-generator',
            event_emitters.sysmon_logs_bucket,
            event_emitters.sysmon_logs_topic,
            event_emitters.unid_subgraphs_generated_bucket
        );


        new NodeIdentityMapper(
            this,
            'node-identity-mapper',
            event_emitters.identity_mappings_bucket,
            event_emitters.identity_mappings_topic,
            history_db.db,
            network.grapl_vpc
        );

        new NodeIdentifier(
            this,
            'node-identifier',
            event_emitters.unid_subgraphs_generated_bucket,
            event_emitters.unid_subgraphs_generated_topic,
            event_emitters.subgraphs_generated_bucket,
            history_db.db,
            network.grapl_vpc
        );

        new GraphMerger(
            this,
            'graph-merger',
            event_emitters.subgraphs_generated_bucket,
            event_emitters.subgraphs_generated_topic,
            event_emitters.subgraph_merged_topic,
            network.grapl_vpc
        );

        //
        // new EngagementCreationService(
        //     this,
        //     'engagement-creation-service',
        //     event_emitters.incident_topic,
        //     network.grapl_vpc
        // );
    }
}

new Grapl().run();
//
// cdk deploy vpcs-stack && \
// cdk deploy event-emitters-stack && \
// cdk deploy history-db-stack && \
// cdk deploy generic-subgraph-generator-stack && \
// cdk deploy node-identifier-stack && \
// cdk deploy node-identity-mapper-stack && \
// cdk deploy graph-merger-stack && \
// cdk deploy word-macro-analyzer-stack && \
// cdk deploy engagement-creation-service-stack

//
// cdk diff vpcs-stack && \
// cdk diff event-emitters-stack && \
// cdk diff history-db-stack && \
// cdk diff generic-subgraph-generator-stack && \
// cdk diff node-identifier-stack && \
// cdk diff node-identity-mapper-stack && \
// cdk diff graph-merger-stack && \
// cdk diff word-macro-analyzer-stack && \
// cdk diff engagement-creation-service-stack