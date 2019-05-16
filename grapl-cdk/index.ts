import route53 = require('@aws-cdk/aws-route53');
import { PrivateHostedZone } from '@aws-cdk/aws-route53';
import { SqsEventSource } from '@aws-cdk/aws-lambda-event-sources';
import cdk = require('@aws-cdk/cdk');
import s3 = require('@aws-cdk/aws-s3');
import sns = require('@aws-cdk/aws-sns');
import sqs = require('@aws-cdk/aws-sqs');
import ec2 = require('@aws-cdk/aws-ec2');
import servicediscovery = require('@aws-cdk/aws-servicediscovery');
import lambda = require('@aws-cdk/aws-lambda');
import iam = require('@aws-cdk/aws-iam');
import {VpcNetwork, IVpcNetwork, TcpPortRange} from "@aws-cdk/aws-ec2";
import {Bucket, IBucket} from "@aws-cdk/aws-s3";
import {Topic, ITopic} from "@aws-cdk/aws-sns";
import {Runtime} from "@aws-cdk/aws-lambda";
import dynamodb = require('@aws-cdk/aws-dynamodb');
import ecs = require('@aws-cdk/aws-ecs');
import cloudtrail = require('@aws-cdk/aws-cloudtrail');
import {BaseService, NamespaceType, NetworkMode} from '@aws-cdk/aws-ecs';

const env = require('node-env-file');

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

    constructor(stack: cdk.Stack, name: string, environment?: any, vpc?: IVpcNetwork, retry_code_name?: string, opt?: any) {
        let runtime = null;
        if (opt && opt.runtime) {
            runtime = opt.runtime
        } else {
            runtime = {name: "provided", supportsInlineCode: true}
        }

        let handler = null;
        if (runtime === Runtime.Python37) {
            handler = `${name}.lambda_handler`
        } else {
            handler = name
        }

        const queues = new Queues(stack, name + '-queue');

        if (environment) {
            environment.QUEUE_URL = queues.queue.queueUrl;
            environment.RUST_BACKTRACE = "1";
        }

        let event_handler = new lambda.Function(
            stack, name, {
                runtime: runtime,
                handler: handler,
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


        if (environment) {
            environment.QUEUE_URL = queues.retry_queue.queueUrl;
        }

        let event_retry_handler = new lambda.Function(
            stack, name + '-retry-handler', {
                runtime: runtime,
                handler: handler,
                code: lambda.Code.asset(`./${retry_code_name}.zip`),
                vpc: vpc,
                environment: environment,
                timeout: 60,
                memorySize: 512,
            }
        );

        event_handler.addEventSource(new SqsEventSource(queues.queue, {batchSize: 1}));
        event_retry_handler.addEventSource(new SqsEventSource(queues.retry_queue, {batchSize: 1}));

        this.queues = queues;
        this.event_handler = event_handler;
        this.event_retry_handler = event_retry_handler;
    }

    readsFrom(bucket: IBucket, with_list?: Boolean) {
        let policy = new iam.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket');

        if(with_list === true) {
            policy.addAction('s3:ListObjects')
        }

        policy.addResource(bucket.bucketArn);

        this.event_handler.addToRolePolicy(policy);
        this.event_retry_handler.addToRolePolicy(policy);

        // TODO: This is adding more permissions than necessary
        bucket.grantRead(this.event_handler.role);
        bucket.grantRead(this.event_retry_handler.role);
    }

    publishesToTopic(publishes_to: ITopic) {
        this.event_handler.addToRolePolicy(new iam.PolicyStatement()
            .addAction('sns:CreateTopic')
            .addResource(publishes_to.topicArn));

        this.event_retry_handler.addToRolePolicy(new iam.PolicyStatement()
            .addAction('sns:CreateTopic')
            .addResource(publishes_to.topicArn));

        publishes_to.grantPublish(this.event_handler.role);
        publishes_to.grantPublish(this.event_retry_handler.role);
    }

    publishesToBucket(publishes_to: IBucket) {

        publishes_to.grantWrite(this.event_handler.role);
        publishes_to.grantWrite(this.event_retry_handler.role);

    }
}


class SessionIdentityCache extends cdk.Stack {
    constructor(parent: cdk.App, vpc_props: ec2.VpcNetworkImportProps) {
        super(parent, 'session-identity-cache-stack');

        // const zone = new route53.PrivateHostedZone(this, 'HostedZone', {
        //     zoneName: 'sessionid.cache',
        //     vpc_props
        // });


    }

}

class EventEmitters extends cdk.Stack {
    raw_logs_bucket: s3.BucketAttributes;
    sysmon_logs_bucket: s3.BucketAttributes;
    identity_mappings_bucket: s3.BucketAttributes;
    unid_subgraphs_generated_bucket: s3.BucketAttributes;
    subgraphs_generated_bucket: s3.BucketAttributes;
    analyzers_bucket: s3.BucketAttributes;
    dispatched_analyzer_bucket: s3.BucketAttributes;
    analyzer_matched_subgraphs_bucket: s3.BucketAttributes;

    incident_topic: sns.TopicAttributes;
    identity_mappings_topic: sns.TopicAttributes;
    raw_logs_topic: sns.TopicAttributes;
    sysmon_logs_topic: sns.TopicAttributes;
    unid_subgraphs_generated_topic: sns.TopicAttributes;
    subgraphs_generated_topic: sns.TopicAttributes;
    subgraph_merged_topic: sns.TopicAttributes;
    dispatched_analyzer_topic: sns.TopicAttributes;
    analyzer_matched_subgraphs_topic: sns.TopicAttributes;
    engagements_created_topic: sns.TopicAttributes;

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

        let analyzers_bucket =
            new s3.Bucket(this, id + '-analyzers-bucket', {
                bucketName: process.env.BUCKET_PREFIX + "-analyzers-bucket"
            });


        let dispatched_analyzer_bucket =
            new s3.Bucket(this, id + '-dispatched-analyzer-bucket', {
                bucketName: process.env.BUCKET_PREFIX + "-dispatched-analyzer-bucket"
            });

        let analyzer_matched_subgraphs_bucket =
            new s3.Bucket(this, id + '-analyzer-matched-subgraphs-bucket', {
                bucketName: process.env.BUCKET_PREFIX + "-analyzer-matched-subgraphs-bucket"
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
        let dispatched_analyzer_topic  =
            new sns.Topic(this, id + '-dispatched-analyzer-topic', {
                topicName: 'dispatched-analyzer-topic'
            });

        let analyzer_matched_subgraphs_topic  =
            new sns.Topic(this, id + '-analyzer-matched-subgraphs-topic', {
                topicName: 'analyzer-matched-subgraphs-topic'
            });
        let engagements_created_topic  =
            new sns.Topic(this, id + '-engagements-created-topic', {
                topicName: 'engagements-created-topic'
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
        dispatched_analyzer_bucket
            .onEvent(s3.EventType.ObjectCreated, dispatched_analyzer_topic);
        analyzer_matched_subgraphs_bucket
            .onEvent(s3.EventType.ObjectCreated, analyzer_matched_subgraphs_topic);

        this.raw_logs_bucket = raw_logs_bucket.export();
        this.sysmon_logs_bucket = sysmon_logs_bucket.export();
        this.identity_mappings_bucket = identity_mappings_bucket.export();
        this.unid_subgraphs_generated_bucket = unid_subgraphs_generated_bucket.export();
        this.subgraphs_generated_bucket = subgraphs_generated_bucket.export();
        this.analyzers_bucket = analyzers_bucket.export();
        this.dispatched_analyzer_bucket = dispatched_analyzer_bucket.export();
        this.analyzer_matched_subgraphs_bucket = analyzer_matched_subgraphs_bucket.export();

        this.incident_topic = incident_topic.export();
        this.raw_logs_topic = raw_logs_topic.export();
        this.sysmon_logs_topic = sysmon_logs_topic.export();
        this.identity_mappings_topic = identity_mappings_topic.export();
        this.unid_subgraphs_generated_topic = unid_subgraphs_generated_topic.export();
        this.subgraphs_generated_topic = subgraphs_generated_topic.export();
        this.subgraph_merged_topic = subgraph_merged_topic.export();
        this.dispatched_analyzer_topic = dispatched_analyzer_topic.export();
        this.analyzer_matched_subgraphs_topic = analyzer_matched_subgraphs_topic.export();
        this.engagements_created_topic = engagements_created_topic.export();
    }
}

class SysmonSubgraphGenerator extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketAttributes,
                subscribes_to_props: sns.TopicAttributes,
                writes_to_props: s3.BucketAttributes,
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.fromBucketAttributes(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.fromTopicAttributes(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const writes_to = Bucket.fromBucketAttributes(
            this,
            'writes_to',
            writes_to_props
        );

        const environment = {
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
        };

        const service = new Service(this, 'sysmon-subgraph-generator', environment);

        service.readsFrom(reads_from);
        subscribes_to.subscribeQueue(service.queues.queue);
        service.publishesToBucket(writes_to);
    }
}


class GenericSubgraphGenerator extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketAttributes,
                subscribes_to_props: sns.TopicAttributes,
                writes_to_props: s3.BucketAttributes,
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.fromBucketAttributes(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.fromTopicAttributes(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const writes_to = Bucket.fromBucketAttributes(
            this,
            'writes_to',
            writes_to_props
        );

        const environment = {
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX
        };

        const service = new Service(this, 'generic-subgraph-generator', environment);

        service.readsFrom(reads_from);
        subscribes_to.subscribeQueue(service.queues.queue);
        service.publishesToBucket(writes_to);
    }
}


class NodeIdentityMapper extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketAttributes,
                subscribes_to_props: sns.TopicAttributes,
                vpc_props: ec2.VpcNetworkImportProps
    ) {
        super(parent, id + '-stack');
        const reads_from = Bucket.fromBucketAttributes(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.fromTopicAttributes(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const vpc = ec2.VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );

        const environment = {
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX
        };

        let service = new Service(this, 'node-identity-mapper', environment, vpc);


        service.readsFrom(reads_from);

        subscribes_to.subscribeQueue(service.queues.queue);

        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
    }
}


class NodeIdentifier extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketAttributes,
                subscribes_to_props: sns.TopicAttributes,
                writes_to_props: s3.BucketAttributes,
                history_db: HistoryDb,
                vpc_props: ec2.VpcNetworkImportProps
    ) {
        super(parent, id + '-stack');
        const reads_from = Bucket.fromBucketAttributes(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.fromTopicAttributes(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const writes_to = Bucket.fromBucketAttributes(
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
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "IDENTITY_CACHE_PEPPER": process.env.IDENTITY_CACHE_PEPPER,
        };

        const service = new Service(this, 'node-identifier', environment, vpc, 'node-identifier-retry-handler');
        service.readsFrom(reads_from);

        history_db.allowReadWrite(service);
        service.publishesToBucket(writes_to);
        subscribes_to.subscribeQueue(service.queues.queue);
        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');

    }
}

class GraphMerger extends cdk.Stack {

    constructor(parent: cdk.App,
                id: string,
                reads_from_props: s3.BucketAttributes,
                subscribes_to_props: sns.TopicAttributes,
                publishes_to_props: sns.TopicAttributes,
                master_graph: DGraphFargate,
                vpc_props: ec2.VpcNetworkImportProps
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.fromBucketAttributes(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.fromTopicAttributes(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const publishes_to = sns.Topic.fromTopicAttributes(
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
            "SUBGRAPH_MERGED_TOPIC_ARN": publishes_to.topicArn,
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "MG_ALPHAS": master_graph.alphaNames.join(",")
        };

        const service = new Service(this, 'graph-merger', environment, vpc);

        // master_graph.addAccessFrom(service);

        service.readsFrom(reads_from);
        service.publishesToTopic(publishes_to);

        subscribes_to.subscribeQueue(service.queues.queue);

        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');

    }
}


class AnalyzerDispatch extends cdk.Stack {

    constructor(parent: cdk.App,
                id: string,
                subscribes_to_props: sns.TopicAttributes,  // The SNS Topic that we must subscribe to our queue
                writes_to_props: s3.BucketAttributes,
                reads_from_props: s3.BucketAttributes,
                vpc_props: ec2.VpcNetworkImportProps
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.fromBucketAttributes(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.fromTopicAttributes(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const writes_to = s3.Bucket.fromBucketAttributes(
            this,
            'publishes_to',
            writes_to_props
        );

        const vpc = ec2.VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );

        const environment = {
            "DISPATCHED_ANALYZER_BUCKET": writes_to.bucketName,
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX
        };

        const service = new Service(this, 'analyzer-dispatcher', environment, vpc);
;
        service.publishesToBucket(writes_to);
        // We need the List capability to find each of the analyzers
        service.readsFrom(reads_from, true);

        subscribes_to.subscribeQueue(service.queues.queue);

        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');
    }
}

class AnalyzerExecutor extends cdk.Stack {

    constructor(parent: cdk.App,
                id: string,
                subscribes_to_props: sns.TopicAttributes,
                reads_analyzers_from_props: s3.BucketAttributes,
                reads_events_from_props: s3.BucketAttributes,
                writes_events_to_props: s3.BucketAttributes,
                master_graph: DGraphFargate,
                vpc_props: ec2.VpcNetworkImportProps
    ) {
        super(parent, id + '-stack');

        const reads_analyzers_from = Bucket.fromBucketAttributes(
            this,
            'reads_analyzers_from',
            reads_analyzers_from_props
        );

        const reads_events_from = Bucket.fromBucketAttributes(
            this,
            'reads_events_from',
            reads_events_from_props
        );

        const subscribes_to = Topic.fromTopicAttributes(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const writes_events_to = Bucket.fromBucketAttributes(
            this,
            'writes_events_to',
            writes_events_to_props
        );

        const vpc = ec2.VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );

        const environment = {
            "ANALYZER_MATCH_BUCKET": writes_events_to.bucketName,
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "MG_ALPHAS": master_graph.alphaNames.join(",")
        };

        const service = new Service(this, 'analyzer-executor', environment, vpc, null, {
            runtime: Runtime.Python37
        });

        // master_graph.addAccessFrom(service);

        service.publishesToBucket(writes_events_to);
        // We need the List capability to find each of the analyzers
        service.readsFrom(reads_analyzers_from, true);
        service.readsFrom(reads_events_from);

        subscribes_to.subscribeQueue(service.queues.queue);

        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');
    }
}

class EngagementCreator extends cdk.Stack {

    constructor(parent: cdk.App,
                id: string,
                reads_from_props: s3.BucketAttributes,
                subscribes_to_props: sns.TopicAttributes,
                publishes_to_props: sns.TopicAttributes,
                master_graph: DGraphFargate,
                engagement_graph: DGraphFargate,
                vpc_props: ec2.VpcNetworkImportProps,
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.fromBucketAttributes(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.fromTopicAttributes(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const publishes_to = sns.Topic.fromTopicAttributes(
            this,
            'publishes_to',
            publishes_to_props
        );

        const vpc = VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );

        const environment = {
            // TODO: I don't think this service reads or writes to S3
            // "BUCKET_PREFIX": process.env.BUCKET_PREFIX
            "MG_ALPHAS": master_graph.alphaNames.join(","),
            "EG_ALPHAS": engagement_graph.alphaNames.join(","),
        };

        const service = new Service(this, 'engagement-creator', environment, vpc, null, {
            runtime: Runtime.Python37
        });

        // master_graph.addAccessFrom(service);
        // engagement_graph.addAccessFrom(service);

        service.readsFrom(reads_from);
        service.publishesToTopic(publishes_to);

        subscribes_to.subscribeQueue(service.queues.queue);

        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');

    }
}


class Networks extends cdk.Stack {
    grapl_vpc: ec2.VpcNetworkImportProps;

    constructor(parent: cdk.App, id: string,) {
        super(parent, id + '-stack');

        const grapl_vpc = new ec2.VpcNetwork(this, 'GraplVPC', {
            natGateways: 1,
            enableDnsHostnames: true,
            enableDnsSupport: true,
        });

        this.grapl_vpc = grapl_vpc.export();
    }
}


class Zero {
    name: string;

    constructor(
        parent: cdk.App,
        stack: cdk.Stack,
        graph: string,
        id: string,
        cluster: ecs.Cluster,
        peer: string,
        idx) {

        const zeroTask = new ecs.FargateTaskDefinition(
            stack,
            id,
            {
                cpu: '256',
                memoryMiB: '2048',
            }
        );

        let command = ["dgraph", "zero", `--my=${id}.${graph}.grapl:5080`,
            "--replicas=3",
            `--idx=${idx}`,
            "--alsologtostderr"];

        if (peer) {
            command.push(`--peer=${peer}.${graph}.grapl:5080`);
        }


        const logDriver = new ecs.AwsLogDriver(stack, graph+id+'LogGroup', {
            streamPrefix: graph+id,
        });

        const container = zeroTask.addContainer(id + 'Container', {

            // --my is our own hostname (graph + id)
            // --peer is the other dgraph zero hostname
            image: ecs.ContainerImage.fromRegistry("dgraph/dgraph"),
            command,
            logging: logDriver
        });

        container.addPortMappings(
            {
                containerPort: 5080,
                hostPort: 5080,
                protocol: ecs.Protocol.Tcp
            },
            {
                containerPort: 6080,
                hostPort: 6080,
                protocol: ecs.Protocol.Tcp
            },
            {
                containerPort: 7080,
                hostPort: 7080,
                protocol: ecs.Protocol.Tcp
            },
            {
                containerPort: 9080,
                hostPort: 9080,
                protocol: ecs.Protocol.Tcp
            },
            {
                containerPort: 8080,
                hostPort: 8080,
                protocol: ecs.Protocol.Tcp
            },
        );


        const zeroService = new ecs.FargateService(stack, id+'Service', {
            cluster,  // Required
            taskDefinition: zeroTask,
        });

        (zeroService as any).enableServiceDiscovery(
            {
                name: id,
                dnsRecordType: servicediscovery.DnsRecordType.A,
                dnsTtlSec: 300,
                // customHealthCheck: {
                //     failureThreshold: 1
                // }
            }

        );

        this.name = `${id}.${graph}.grapl`;

        zeroService.connections.allowFromAnyIPv4(new ec2.TcpAllPorts());
    }
}


class Alpha {
    name: string;

    constructor(
        parent: cdk.App,
        stack: cdk.Stack,
        graph: string,
        id: string,
        cluster: ecs.Cluster,
        zero: string) {

        const alphaTask = new ecs.FargateTaskDefinition(
            stack,
            id,
            {
                cpu: '256',
                memoryMiB: '2048'
            }
        );

        const logDriver = new ecs.AwsLogDriver(stack, graph+id+'LogGroup', {
            streamPrefix: graph+id,
        });

        const container = alphaTask.addContainer(id + graph + 'Container', {
            image: ecs.ContainerImage.fromRegistry("dgraph/dgraph"),
            command: ["dgraph", "alpha", `--my=${id}.${graph}.grapl:7080`,
                "--lru_mb=1024", `--zero=${zero}.${graph}.grapl:5080`,
                "--alsologtostderr"
            ],
            logging: logDriver
        });

        container.addPortMappings(
            {
                containerPort: 5080,
                hostPort: 5080,
                protocol: ecs.Protocol.Tcp
            },
            {
                containerPort: 6080,
                hostPort: 6080,
                protocol: ecs.Protocol.Tcp
            },
            {
                containerPort: 7080,
                hostPort: 7080,
                protocol: ecs.Protocol.Tcp
            },
            {
                containerPort: 9080,
                hostPort: 9080,
                protocol: ecs.Protocol.Tcp
            },
            {
                containerPort: 8080,
                hostPort: 8080,
                protocol: ecs.Protocol.Tcp
            },
        );

        const alphaService = new ecs.FargateService(stack, id+'Service', {
            cluster,  // Required
            taskDefinition: alphaTask
        });

        (alphaService as any).enableServiceDiscovery(
            {
                name: id,
                dnsRecordType: servicediscovery.DnsRecordType.A,
                dnsTtlSec: 300,
            }

        );

        this.name = `${id}.${graph}.grapl`;

        alphaService.connections.allowFromAnyIPv4(new ec2.TcpAllPorts());
    }
}

class DGraphFargate extends cdk.Stack {
    alphaNames: string[];

    constructor(
        parent: cdk.App,
        id: string,
        vpc_props: ec2.VpcNetworkImportProps,
        zeroCount,
        alphaCount
    ) {
        super(parent, id+'-stack');

        const vpc = VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );

        const cluster = new ecs.Cluster(this, id+'-FargateCluster', {
            vpc: vpc
        });


        const namespace = cluster.addDefaultCloudMapNamespace(
            {
                name: id + '.grapl',
                type: NamespaceType.PrivateDns,
                vpc
            }
        );

        const zero0 = new Zero(
            parent,
            this,
            id,
            'zero0',
            cluster,
            null,
            1
        );

        for (let i = 1; i < zeroCount ; i++) {
            const zero0 = new Zero(
                parent,
                this,
                id,
                `zero${i}`,
                cluster,
                'zero0',
                1
            );
        }


        this.alphaNames = [];

        for (let i = 0; i < alphaCount ; i++) {

            const alpha = new Alpha(
                parent,
                this,
                id,
                `alpha${i}`, // increment for each alpha
                cluster,
                "zero0"
            );

            this.alphaNames.push(alpha.name);
        }

    }
}

class HistoryDb extends cdk.Stack {

    proc_history: dynamodb.Table;
    file_history: dynamodb.Table;
    asset_history: dynamodb.Table;

    constructor(parent: cdk.App,
                id: string,
    ) {
        super(parent, id + '-stack');

        this.proc_history = new dynamodb.Table(this, 'process_history_table', {
            tableName: "process_history_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.String
            },
            sortKey: {
                name: 'create_time',
                type: dynamodb.AttributeType.Number
            },
            billingMode: dynamodb.BillingMode.PayPerRequest,
        });

        this.file_history = new dynamodb.Table(this, 'file_history_table', {
            tableName: "file_history_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.String
            },
            sortKey: {
                name: 'create_time',
                type: dynamodb.AttributeType.Number
            },
            billingMode: dynamodb.BillingMode.PayPerRequest,
        });

        this.asset_history = new dynamodb.Table(this, 'asset_id_mappings', {
            tableName: "asset_id_mappings",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.String
            },
            sortKey: {
                name: 'c_timestamp',
                type: dynamodb.AttributeType.Number
            },
            billingMode: dynamodb.BillingMode.PayPerRequest,
        });
    }

    allowReadWrite(service: Service) {
        this.proc_history.grantReadWriteData(service.event_handler.role);
        this.file_history.grantReadWriteData(service.event_handler.role);
        this.asset_history.grantReadWriteData(service.event_handler.role);

        this.proc_history.grantReadWriteData(service.event_retry_handler.role);
        this.file_history.grantReadWriteData(service.event_retry_handler.role);
        this.asset_history.grantReadWriteData(service.event_retry_handler.role);
    }
}

class Grapl extends cdk.App {
    constructor() {
        super();

        env(__dirname + '/.env');

        const mgZeroCount = Number(process.env.MG_ZEROS_COUNT) || 3;
        const mgAlphaCount = Number(process.env.MG_ALPHAS_COUNT) || 5;
        const egZeroCount = Number(process.env.EG_ZEROS_COUNT) || 3;
        const egAlphaCount = Number(process.env.EG_ALPHAS_COUNT) || 5;

        let event_emitters = new EventEmitters(this, 'grapl-event-emitters');

        const network = new Networks(this, 'graplvpcs');

        const history_db = new HistoryDb(
            this,
            'graplhistorydb',
        );

        const master_graph = new DGraphFargate(
            this,
            'mastergraphcluster',
            network.grapl_vpc,
            mgZeroCount,
            mgAlphaCount,
    );

        const engagement_graph = new DGraphFargate(
            this,
            'engagementgraphcluster',
            network.grapl_vpc,
            egZeroCount,
            egAlphaCount,
    );

        // TODO: Move subgraph generators to their own VPC
        new GenericSubgraphGenerator(
            this,
            'grapl-generic-subgraph-generator',
            event_emitters.raw_logs_bucket,
            event_emitters.raw_logs_topic,
            event_emitters.unid_subgraphs_generated_bucket
        );

        new SysmonSubgraphGenerator(
            this,
            'grapl-sysmon-subgraph-generator',
            event_emitters.sysmon_logs_bucket,
            event_emitters.sysmon_logs_topic,
            event_emitters.unid_subgraphs_generated_bucket
        );


        new NodeIdentityMapper(
            this,
            'grapl-node-identity-mapper',
            event_emitters.identity_mappings_bucket,
            event_emitters.identity_mappings_topic,
            network.grapl_vpc
        );

        new NodeIdentifier(
            this,
            'grapl-node-identifier',
            event_emitters.unid_subgraphs_generated_bucket,
            event_emitters.unid_subgraphs_generated_topic,
            event_emitters.subgraphs_generated_bucket,
            history_db,
            network.grapl_vpc
        );

        new GraphMerger(
            this,
            'grapl-graph-merger',
            event_emitters.subgraphs_generated_bucket,
            event_emitters.subgraphs_generated_topic,
            event_emitters.subgraph_merged_topic,
            master_graph,
            network.grapl_vpc
        );

        new AnalyzerDispatch(
            this,
            'grapl-analyzer-dispatcher',
            event_emitters.subgraph_merged_topic,
            event_emitters.dispatched_analyzer_bucket,
            event_emitters.analyzers_bucket,
            network.grapl_vpc
        );

        new AnalyzerExecutor(
            this,
            'grapl-analyzer-executor',
            event_emitters.dispatched_analyzer_topic,
            event_emitters.analyzers_bucket,
            event_emitters.dispatched_analyzer_bucket,
            event_emitters.analyzer_matched_subgraphs_bucket,
            master_graph,
            network.grapl_vpc
        );

        new EngagementCreator(
            this,
            'grapl-engagement-creator',
            event_emitters.analyzer_matched_subgraphs_bucket,
            event_emitters.analyzer_matched_subgraphs_topic,
            event_emitters.engagements_created_topic,
            master_graph,
            engagement_graph,
            network.grapl_vpc
        );
    }
}

new Grapl().run();

// cdk deploy graplvpcs-stack && \
// cdk deploy graplhistorydb-stack && \
// cdk deploy grapl-event-emitters-stack && \
// cdk deploy graplmastergraph-stack && \
// cdk deploy graplengagementgraph-stack && \
// cdk deploy grapl-generic-subgraph-generator-stack && \
// cdk deploy grapl-sysmon-subgraph-generator-stack && \
// cdk deploy grapl-node-identity-mapper-stack && \
// cdk deploy grapl-node-identifier-stack && \
// cdk deploy grapl-graph-merger-stack && \
// cdk deploy grapl-analyzer-dispatcher-stack && \
// cdk deploy grapl-analyzer-executor-stack && \
// cdk deploy grapl-engagement-creator-stack


