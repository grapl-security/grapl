import cdk = require('@aws-cdk/cdk');
import s3 = require('@aws-cdk/aws-s3');
import sns = require('@aws-cdk/aws-sns');
import sqs = require('@aws-cdk/aws-sqs');
import ec2 = require('@aws-cdk/aws-ec2');
import rds = require('@aws-cdk/aws-rds');
import lambda = require('@aws-cdk/aws-lambda');
import {cloudformation, PrefixList, SubnetType, TcpAllPorts, VpcNetwork} from "@aws-cdk/aws-ec2";
import {Stack, Token} from "@aws-cdk/cdk";
import {Bucket} from "@aws-cdk/aws-s3";
import {Topic} from "@aws-cdk/aws-sns";
import {DatabaseCluster, DatabaseClusterRefProps} from "@aws-cdk/aws-rds";
import SecurityGroupEgressResource = cloudformation.SecurityGroupEgressResource;

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
                ec2.InstanceSize.Small
            ),
            vpc: vpc,
            vpcPlacement: {
                subnetsToUse: SubnetType.Private
            }
        }
    });

}

function subscribe_lambda_to_queue(stack: cdk.Stack, id: string, fn: lambda.Function, queue: sqs.Queue) {

    // TODO: Build the S3 Endpoint and allow traffic only through that endpoint
    new lambda.cloudformation.EventSourceMappingResource(stack, id + 'Events', {
        functionName: fn.functionName,
        eventSourceArn: queue.queueArn
    });

    fn.addToRolePolicy(new cdk.PolicyStatement()
        .addAction('sqs:ReceiveMessage')
        .addAction('sqs:DeleteMessage')
        .addAction('sqs:GetQueueAttributes')
        .addAction('sqs:*')
        .addResource(queue.queueArn));
}

class EventEmitters extends cdk.Stack {
    raw_logs_bucket: s3.BucketRefProps;
    unid_subgraphs_generated_bucket: s3.BucketRefProps;
    subgraphs_generated_bucket: s3.BucketRefProps;

    incident_topic: sns.TopicRefProps;
    raw_logs_topic: sns.TopicRefProps;
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
        unid_subgraphs_generated_bucket
            .onEvent(s3.EventType.ObjectCreated, unid_subgraphs_generated_topic);
        subgraphs_generated_bucket
            .onEvent(s3.EventType.ObjectCreated, subgraphs_generated_topic);

        this.raw_logs_bucket = raw_logs_bucket.export();
        this.unid_subgraphs_generated_bucket = unid_subgraphs_generated_bucket.export();
        this.subgraphs_generated_bucket = subgraphs_generated_bucket.export();

        this.incident_topic = incident_topic.export();
        this.raw_logs_topic = raw_logs_topic.export();
        this.unid_subgraphs_generated_topic = unid_subgraphs_generated_topic.export();
        this.subgraphs_generated_topic = subgraphs_generated_topic.export();
        this.subgraph_merged_topic = subgraph_merged_topic.export();
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

        // Generic subgraph generator
        let generic_subgraph_generator = new lambda.Function(
            this, 'generic-subgraph-generator', {
                runtime: lambda.Runtime.Go1x,
                handler: 'generic-subgraph-generator',
                code: lambda.Code.file('./generic-subgraph-generator.zip'),
                environment: {
                    "BUCKET_PREFIX": process.env.BUCKET_PREFIX
                }
            }
        );

        generic_subgraph_generator.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addAction('s3:ActionList')
            .addResource(reads_from.bucketArn));


        reads_from.grantRead(generic_subgraph_generator.role);



        let generic_subgraph_generator_queue =
            new sqs.Queue(this, 'generic-subgraph-generator-queue');

        subscribe_lambda_to_queue(this, 'genericSubgraphGenerator', generic_subgraph_generator, generic_subgraph_generator_queue);
        event_producer.subscribeQueue(generic_subgraph_generator_queue);

        writes_to.grantWrite(generic_subgraph_generator.role);
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

        // Add retry handler for node identifier retry handler
        let node_identifier_rth_dead_letter_queue =
            new sqs.Queue(this, 'node-identifier-rth-dead-letter-queue');

        let node_identifier_retry_handler = new lambda.Function(
            this, 'node-identifier-retry-handler', {
                runtime: lambda.Runtime.Go1x,
                handler: 'node-identifier-retry-handler',
                code: lambda.Code.file('./node-identifier-retry-handler.zip'),
                vpc: vpc,
                environment: {
                    "HISTORY_DB_USERNAME": process.env.HISTORY_DB_USERNAME,
                    "HISTORY_DB_PASSWORD": process.env.HISTORY_DB_PASSWORD,
                    "BUCKET_PREFIX": process.env.BUCKET_PREFIX
                }
            }
        );

        node_identifier_retry_handler.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addAction('s3:ActionList')
            .addResource(reads_from.bucketArn));

        reads_from.grantRead(node_identifier_retry_handler.role);

        let node_identifier_retry_handler_queue =
            new sqs.Queue(this, 'node-identifier-retry-handler-queue', {
                deadLetterQueue: {queue: node_identifier_rth_dead_letter_queue, maxReceiveCount: 5}
            });

        subscribe_lambda_to_queue(
            this,
            'nodeIdentifierRetryHandler',
            node_identifier_retry_handler,
            node_identifier_retry_handler_queue
        );

        // Node Identifier
        let node_identifier = new lambda.Function(
            this, 'node-identifier', {
                runtime: lambda.Runtime.Go1x,
                handler: 'node-identifier',
                code: lambda.Code.file('./node-identifier.zip'),
                vpc: vpc,
                environment: {
                    "HISTORY_DB_USERNAME": process.env.HISTORY_DB_USERNAME,
                    "HISTORY_DB_PASSWORD": process.env.HISTORY_DB_PASSWORD,
                    "BUCKET_PREFIX": process.env.BUCKET_PREFIX
                }
            }
        );

        node_identifier.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addAction('s3:ActionList')
            .addResource(reads_from.bucketArn));

        reads_from.grantRead(node_identifier.role);

        let node_identifier_queue =
            new sqs.Queue(this, 'node-identifier-queue', {
                deadLetterQueue: {queue: node_identifier_retry_handler_queue, maxReceiveCount: 5}
            });

        subscribe_lambda_to_queue(this, 'nodeIdentifier', node_identifier, node_identifier_queue);

        history_db.connections.allowDefaultPortFrom(
          node_identifier,
          'node-identifier-history-db'
        );

        history_db.connections.allowDefaultPortFrom(
            node_identifier_retry_handler,
            'node-identifier-retry-handler-history-db'
        );

        event_producer.subscribeQueue(node_identifier_queue);

        writes_to.grantWrite(node_identifier.role);
        writes_to.grantWrite(node_identifier_retry_handler.role);

        node_identifier.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');
        node_identifier_retry_handler.connections.allowToAnyIPv4(new ec2.TcpPort(443), 'Allow outbound to S3');

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

        let graph_merger = new lambda.Function(
            this, 'graph-merger', {
                runtime: lambda.Runtime.Go1x,
                handler: 'graph-merger',
                code: lambda.Code.file('./graph-merger.zip'),
                vpc: vpc,
                environment: {
                    "GRAPH_MERGER": process.env.GRAPH_MERGER_READ_BUCKET,
                    "GRAPH_MERGER_WRITE_TOPIC": process.env.GRAPH_MERGER_WRITE_TOPIC,
                    "BUCKET_PREFIX": process.env.BUCKET_PREFIX
                }
            }
        );

        let graph_merger_queue =
            new sqs.Queue(this, 'graph-merger-queue');

        subscribe_lambda_to_queue(this, 'graphMerger', graph_merger, graph_merger_queue);

        graph_merger.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addAction('s3:ActionList')
            .addResource(reads_from.bucketArn));


        graph_merger.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('sns:CreateTopic')
            .addResource(publishes_to.topicArn));

        reads_from.grantRead(graph_merger.role);
        event_producer.subscribeQueue(graph_merger_queue);
        publishes_to.grantPublish(graph_merger.role);
        graph_merger.connections.allowToAnyIPv4(new ec2.TcpAllPorts(), 'Allow outbound to S3');

    }
}

class WordMacroAnalyzer extends cdk.Stack {

    constructor(parent: cdk.App,
                id: string,
                event_producer_props: sns.TopicRefProps,
                publishes_to_props: sns.TopicRefProps,
                vpc_props: ec2.VpcNetworkRefProps
    ) {
        super(parent, id + '-stack');


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
        let word_macro_analyzer = new lambda.Function(
            this, 'word-macro-analyzer', {
                runtime: lambda.Runtime.Python36,
                handler: 'word-macro-analyzer.lambda_handler',
                code: lambda.Code.file('./word-macro-analyzer.zip'),
                vpc: vpc,
            }
        );

        let word_macro_analyzer_queue =
            new sqs.Queue(this, 'word-macro-analyzer-queue');

        subscribe_lambda_to_queue(
            this,
            'wordMacroAnalyzer',
            word_macro_analyzer,
            word_macro_analyzer_queue
        );

        event_producer.subscribeQueue(word_macro_analyzer_queue);

        publishes_to.grantPublish(word_macro_analyzer.role);
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
                code: lambda.Code.file('./engagement-creation-service.zip'),
                vpc: vpc
            }
        );

        let engagement_creation_service_queue =
            new sqs.Queue(this, 'engagement-creation-service-queue');

        subscribe_lambda_to_queue(this, 'engagementCreator', engagement_creation_service, engagement_creation_service_queue);
        event_producer.subscribeQueue(engagement_creation_service_queue);
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
    constructor(argv: string[]) {
        super(argv);

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

        new WordMacroAnalyzer(
            this,
            'word-macro-analyzer',
            event_emitters.subgraph_merged_topic,
            event_emitters.incident_topic,
            network.grapl_vpc
        );

        new EngagementCreationService(
            this,
            'engagement-creation-service',
            event_emitters.incident_topic,
            network.grapl_vpc
        );
    }
}

process.stdout.write(new Grapl(process.argv).run());
//
// cdk deploy vpcs-stack && \
// cdk deploy event-emitters-stack && \
// cdk deploy history-db-stack && \
// cdk deploy generic-subgraph-generator-stack && \
// cdk deploy node-identifier-stack && \
// cdk deploy graph-merger-stack && \
// cdk deploy word-macro-analyzer-stack && \
// cdk deploy engagement-creation-service-stack
