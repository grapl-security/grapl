
import route53 = require('@aws-cdk/aws-route53');
import { PrivateHostedZone } from '@aws-cdk/aws-route53';
import { SqsEventSource } from '@aws-cdk/aws-lambda-event-sources';
import cdk = require('@aws-cdk/cdk');
import s3 = require('@aws-cdk/aws-s3');
import sns = require('@aws-cdk/aws-sns');
import sqs = require('@aws-cdk/aws-sqs');
import ec2 = require('@aws-cdk/aws-ec2');
import rds = require('@aws-cdk/aws-rds');
import lambda = require('@aws-cdk/aws-lambda');
import iam = require('@aws-cdk/aws-iam');
import {SubnetType, VpcNetwork, IVpcNetwork} from "@aws-cdk/aws-ec2";
import {Token} from "@aws-cdk/cdk";
import {Bucket, BucketImportProps, IBucket} from "@aws-cdk/aws-s3";
import {Topic, ITopic} from "@aws-cdk/aws-sns";
import {DatabaseCluster} from "@aws-cdk/aws-rds";
import {Runtime} from "@aws-cdk/aws-lambda";

const env = require('node-env-file');

function get_history_db(stack: cdk.Stack, vpc: ec2.IVpcNetwork, username: Token, password: Token): DatabaseCluster {
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
    raw_logs_bucket: s3.BucketImportProps;
    sysmon_logs_bucket: s3.BucketImportProps;
    identity_mappings_bucket: s3.BucketImportProps;
    unid_subgraphs_generated_bucket: s3.BucketImportProps;
    subgraphs_generated_bucket: s3.BucketImportProps;
    analyzers_bucket: s3.BucketImportProps;
    dispatched_analyzer_bucket: s3.BucketImportProps;
    analyzer_matched_subgraphs_bucket: s3.BucketImportProps;

    incident_topic: sns.TopicImportProps;
    identity_mappings_topic: sns.TopicImportProps;
    raw_logs_topic: sns.TopicImportProps;
    sysmon_logs_topic: sns.TopicImportProps;
    unid_subgraphs_generated_topic: sns.TopicImportProps;
    subgraphs_generated_topic: sns.TopicImportProps;
    subgraph_merged_topic: sns.TopicImportProps;
    dispatched_analyzer_topic: sns.TopicImportProps;
    analyzer_matched_subgraphs_topic: sns.TopicImportProps;
    engagements_created_topic: sns.TopicImportProps;

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
                reads_from_props: s3.BucketImportProps,
                subscribes_to_props: sns.TopicImportProps,
                writes_to_props: s3.BucketImportProps,
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.import(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const writes_to = Bucket.import(
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
                reads_from_props: s3.BucketImportProps,
                subscribes_to_props: sns.TopicImportProps,
                writes_to_props: s3.BucketImportProps,
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.import(
            this,
            'subscribes_to',
            subscribes_to_props
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
        subscribes_to.subscribeQueue(service.queues.queue);
        service.publishesToBucket(writes_to);
    }
}


class NodeIdentityMapper extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketImportProps,
                subscribes_to_props: sns.TopicImportProps,
                history_db_props: rds.DatabaseClusterImportProps,
                vpc_props: ec2.VpcNetworkImportProps
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

        const subscribes_to = Topic.import(
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
            "HISTORY_DB_USERNAME": process.env.HISTORY_DB_USERNAME,
            "HISTORY_DB_PASSWORD": process.env.HISTORY_DB_PASSWORD,
            "HISTORY_DB_ADDRESS": "db.historydb", // TODO: Derive this
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
        subscribes_to.subscribeQueue(service.queues.queue);

        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
    }
}


class NodeIdentifier extends cdk.Stack {

    constructor(parent: cdk.App, id: string,
                reads_from_props: s3.BucketImportProps,
                subscribes_to_props: sns.TopicImportProps,
                writes_to_props: s3.BucketImportProps,
                history_db_props: rds.DatabaseClusterImportProps,
                vpc_props: ec2.VpcNetworkImportProps
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

        const subscribes_to = Topic.import(
            this,
            'subscribes_to',
            subscribes_to_props
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
            "HISTORY_DB_ADDRESS": "db.historydb", // TODO: Derive this
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

        service.publishesToBucket(writes_to);
        subscribes_to.subscribeQueue(service.queues.queue);
        service.event_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');

    }
}

class GraphMerger extends cdk.Stack {

    constructor(parent: cdk.App,
                id: string,
                reads_from_props: s3.BucketImportProps,
                subscribes_to_props: sns.TopicImportProps,
                publishes_to_props: sns.TopicImportProps,
                master_graph: GraphDB,
                vpc_props: ec2.VpcNetworkImportProps
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.import(
            this,
            'subscribes_to',
            subscribes_to_props
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
            "SUBGRAPH_MERGED_TOPIC_ARN": publishes_to.topicArn,
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX
        };

        const service = new Service(this, 'graph-merger', environment, vpc);

        master_graph.addAccessFrom(service);

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
                subscribes_to_props: sns.TopicImportProps,  // The SNS Topic that we must subscribe to our queue
                writes_to_props: s3.BucketImportProps,
                reads_from_props: s3.BucketImportProps,
                vpc_props: ec2.VpcNetworkImportProps
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.import(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const writes_to = s3.Bucket.import(
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
                subscribes_to_props: sns.TopicImportProps,
                reads_analyzers_from_props: s3.BucketImportProps,
                reads_events_from_props: s3.BucketImportProps,
                writes_events_to_props: s3.BucketImportProps,
                master_graph: GraphDB,
                vpc_props: ec2.VpcNetworkImportProps
    ) {
        super(parent, id + '-stack');

        const reads_analyzers_from = Bucket.import(
            this,
            'reads_analyzers_from',
            reads_analyzers_from_props
        );

        const reads_events_from = Bucket.import(
            this,
            'reads_events_from',
            reads_events_from_props
        );

        const subscribes_to = Topic.import(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const writes_events_to = Bucket.import(
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
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX
        };

        const service = new Service(this, 'analyzer-executor', environment, vpc, null, {
            runtime: Runtime.Python37
        });

        master_graph.addAccessFrom(service);

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
                reads_from_props: s3.BucketImportProps,
                subscribes_to_props: sns.TopicImportProps,
                publishes_to_props: sns.TopicImportProps,
                master_graph: GraphDB,
                engagement_graph: GraphDB,
                vpc_props: ec2.VpcNetworkImportProps,
    ) {
        super(parent, id + '-stack');

        const reads_from = Bucket.import(
            this,
            'reads_from',
            reads_from_props
        );

        const subscribes_to = Topic.import(
            this,
            'subscribes_to',
            subscribes_to_props
        );

        const publishes_to = sns.Topic.import(
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
        };

        const service = new Service(this, 'engagement-creator', environment, vpc, null, {
            runtime: Runtime.Python37
        });

        master_graph.addAccessFrom(service);
        engagement_graph.addAccessFrom(service);

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
            natGateways: 2,
            enableDnsHostnames: true,
            enableDnsSupport: true,
        });

        this.grapl_vpc = grapl_vpc.export();
    }
}

class GraphDB extends cdk.Stack {
    vpc: IVpcNetwork;
    graph_security_group: ec2.SecurityGroup;
    id: string;

    constructor(parent: cdk.App, id: string, vpc_props: ec2.VpcNetworkImportProps, dnsName: string, options?: any) {
        super(parent, id + '-stack');
        this.id = id;

        const vpc = VpcNetwork.import(
            this,
            'vpc',
            vpc_props
        );

        this.vpc = vpc;

        let graph_security_group = new ec2.SecurityGroup(this, this.id +'-security-group',
            {vpc:this.vpc}
        );

        const db = new ec2.CfnInstance(this, id + "Ec2", {
            instanceType: new ec2.InstanceTypePair(ec2.InstanceClass.M3, ec2.InstanceSize.Medium).toString(),
            securityGroupIds: [graph_security_group.securityGroupId],
            subnetId: vpc.publicSubnets[0].subnetId,
            imageId: "ami-0ac019f4fcb7cb7e6",
            keyName: process.env.GRAPH_DB_KEY_NAME
        });

        const zone = new PrivateHostedZone(this, id + '-hosted-zone', {
            zoneName: id,
            vpc
        });

        new route53.CnameRecord(
            this, id, {
                zone,
                recordName: 'db.' + id,
                recordValue: db.instancePublicDnsName
            }
        );

        if (options.allow_all_ssh) {
            graph_security_group.addIngressRule(new ec2.AnyIPv4(), new ec2.TcpAllPorts());
        }

        this.graph_security_group = graph_security_group;
    }

    addAccessFrom(service: Service) {
        // TODO: Don't allow all ports


        for (const security_group of service.event_retry_handler.connections.securityGroups) {

            // this.graph_security_group.addIngressRule(service.event_handler.sec, new ec2.TcpAllPorts());
            security_group.addEgressRule(this.graph_security_group, new ec2.TcpAllPorts())
        }

        for (const security_group of service.event_handler.connections.securityGroups) {
            // this.graph_security_group.addIngressRule(security_group, new ec2.TcpAllPorts());
            security_group.addEgressRule(this.graph_security_group, new ec2.TcpAllPorts())
        }
    }
}


class HistoryDb extends cdk.Stack {

    db: rds.DatabaseClusterImportProps;

    constructor(parent: cdk.App,
                id: string,
                grapl_vpc_props: ec2.VpcNetworkImportProps,
                username: cdk.Token,
                password: cdk.Token
    ) {
        super(parent, id + '-stack');

        const vpc = ec2.VpcNetwork.import(
            this,
            'vpc',
            grapl_vpc_props
        );


        const db = get_history_db(this, vpc, username, password);

        // TODO: This should be dynamic, based on services that require access, and only for
        //       the required ports
        db.connections.allowFromAnyIPv4(new ec2.TcpAllPorts());

        const zone = new PrivateHostedZone(this, id + '-hosted-zone', {
            zoneName: id,
            vpc
        });

        new route53.CnameRecord(
            this, 'historydb', {
                zone,
                recordName: 'db.historydb',
                recordValue: db.clusterEndpoint.hostname
            }
        );
        this.db = db.export();
    }
}

class Grapl extends cdk.App {
    constructor() {
        super();

        const env_file = env(__dirname + '/.env');

        const network = new Networks(this, 'vpcs');

        const history_db = new HistoryDb(
            this,
            'historydb',
            network.grapl_vpc,
            new cdk.Token(process.env.HISTORY_DB_USERNAME),
            new cdk.Token(process.env.HISTORY_DB_PASSWORD)
        );

        let event_emitters = new EventEmitters(this, 'event-emitters');

        const master_graph = new GraphDB(
            this,
            'mastergraph',
            network.grapl_vpc,
            "ec2-3-87-59-85.compute-1.amazonaws.com", {
            allow_all_ssh: true
        });

        const engagement_graph = new GraphDB(
            this,
            'engagementgraph',
            network.grapl_vpc,
            "ip-10-0-190-70.ec2.internal", {
            allow_all_ssh: true
        });

        // TODO: Move subgraph generators to their own VPC
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
            master_graph,
            network.grapl_vpc
        );

        new AnalyzerDispatch(
            this,
            'analyzer-dispatcher',
            event_emitters.subgraph_merged_topic,
            event_emitters.dispatched_analyzer_bucket,
            event_emitters.analyzers_bucket,
            network.grapl_vpc
        );

        new AnalyzerExecutor(
            this,
            'analyzer-executor',
            event_emitters.dispatched_analyzer_topic,
            event_emitters.analyzers_bucket,
            event_emitters.dispatched_analyzer_bucket,
            event_emitters.analyzer_matched_subgraphs_bucket,
            master_graph,
            network.grapl_vpc
        );

        new EngagementCreator(
            this,
            'engagement-creator',
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