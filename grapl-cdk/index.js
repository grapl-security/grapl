"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const cdk = require("@aws-cdk/cdk");
const s3 = require("@aws-cdk/aws-s3");
const sns = require("@aws-cdk/aws-sns");
const sqs = require("@aws-cdk/aws-sqs");
const ec2 = require("@aws-cdk/aws-ec2");
const rds = require("@aws-cdk/aws-rds");
const lambda = require("@aws-cdk/aws-lambda");
const aws_ec2_1 = require("@aws-cdk/aws-ec2");
function get_history_db(stack, vpc, username, password) {
    return new rds.DatabaseCluster(stack, 'HistoryDb', {
        defaultDatabaseName: 'historydb',
        masterUser: {
            username: username.toString(),
            password: password.toString(),
        },
        engine: rds.DatabaseClusterEngine.Aurora,
        instanceProps: {
            instanceType: new ec2.InstanceTypePair(ec2.InstanceClass.Burstable2, ec2.InstanceSize.Small),
            vpc: vpc,
            vpcPlacement: {
                subnetsToUse: aws_ec2_1.SubnetType.Private
            }
        }
    });
}
function subscribe_lambda_to_queue(stack, id, fn, queue) {
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
//
//
// class S3SnsSqs {
//     public bucket: s3.Bucket;
//     public sns_topic: sns.Topic;
//     public queue: sqs.Queue;
//     constructor(name: string,
//                 stack: cdk.Stack,
//                 vpc?: ec2.VpcNetworkRef,
//                 dead_letter_queue?: sqs.Queue,
//                 max_receive_count?: number
//     ) {
//
//         this.bucket = new s3.Bucket(stack, name + '-bucket');
//         this.sns_topic = new sns.Topic(stack, name + '-topic');
//
//         if (dead_letter_queue != null) {
//             this.queue = new sqs.Queue(stack, name + '-retry-handler-queue', {
//                 deadLetterQueue: { queue: dead_letter_queue, maxReceiveCount: max_receive_count }
//             });
//         } else {
//             this.queue = new sqs.Queue(stack, name + '-retry-handler-queue');
//         }
//     }
// }
//
// function word_macro_analyzer(stack: cdk.Stack, vpc: ec2.VpcNetwork) {
//     const events = new S3SnsSqs('word-macro-analyzer', stack, vpc);
//     const analyzer = new SqsLambda('generic-subgraph-generator', stack, events.queue, vpc);
//
//
// }
//
// class SqsLambda {
//     public fn: lambda.Function;
//
//     constructor(
//         name: string,
//         stack: cdk.Stack,
//         queue: sqs.QueueRef,
//         vpc?: ec2.VpcNetworkRef,
//         runtime?: lambda.Runtime
//     ) {
//
//         runtime = runtime || lambda.Runtime.Go1x;
//
//         this.fn = new lambda.Function(
//             stack,
//             name,
//             {
//                 runtime: runtime,
//                 handler: '${name}',
//                 code: lambda.Code.file(`./${name}zip`),
//                 vpc: vpc
//             }
//         );
//     }
// }
// function new_lambda(
//     stack: cdk.Stack,
//     name: string,
//     queue: sqs.Queue,
//     reads_from: s3.BucketRef,
//     writes_to: s3.BucketRef
// ): lambda.FunctionRef {
//
//
//
//
//     return null;
//
// }
class GraplStack extends cdk.Stack {
    constructor(parent, id) {
        super(parent, id + '-stack');
        let history_db_vpc = new ec2.VpcNetwork(this, 'HistoryVPC');
        let dgraph_vpc = new ec2.VpcNetwork(this, 'DGraphVPC');
        const username = new cdk.Token('username');
        const password = new cdk.Token('passwd1234567890');
        let db = get_history_db(this, history_db_vpc, username, password);
        // S3 buckets
        let raw_logs_bucket = new s3.Bucket(this, id + '-raw-log-bucket');
        let unid_subgraphs_generated_bucket = new s3.Bucket(this, id + '-unid-subgraphs-generated-bucket');
        let subgraphs_generated_bucket = new s3.Bucket(this, id + '-subgraphs-generated-bucket');
        // SNS Topics
        let incident_topic = new sns.Topic(this, id + '-incident-topic');
        let raw_logs_topic = new sns.Topic(this, '-raw-log-topic');
        let unid_subgraphs_generated_topic = new sns.Topic(this, '-unid-subgraphs-generated-topic');
        let subgraphs_generated_topic = new sns.Topic(this, '-subgraphs-generated-topic');
        let subgraph_merged_topic = new sns.Topic(this, id + '-subgraphs-merged-topic');
        // S3 -> SNS Events
        raw_logs_bucket
            .onEvent(s3.EventType.ObjectCreated, raw_logs_topic);
        unid_subgraphs_generated_bucket
            .onEvent(s3.EventType.ObjectCreated, unid_subgraphs_generated_topic);
        subgraphs_generated_bucket
            .onEvent(s3.EventType.ObjectCreated, subgraphs_generated_topic);
        // Services and Queues
        // Node identity mapper
        let node_identity_mapper = new lambda.Function(this, 'node-identity-mapper', {
            runtime: lambda.Runtime.Go1x,
            handler: 'node-identity-mapper',
            code: lambda.Code.file('./node-identity-mapper.zip'),
            vpc: history_db_vpc
        });
        //
        node_identity_mapper.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addAction('s3:ActionList')
            .addResource(unid_subgraphs_generated_bucket.bucketArn));
        unid_subgraphs_generated_bucket.grantRead(node_identity_mapper.role);
        let node_identity_mapper_queue = new sqs.Queue(this, 'node-identity-mapper-queue');
        subscribe_lambda_to_queue(this, 'nodeIdentityMapper', node_identity_mapper, node_identity_mapper_queue);
        // Generic subgraph generator
        let generic_subgraph_generator = new lambda.Function(this, 'generic-subgraph-generator', {
            runtime: lambda.Runtime.Go1x,
            handler: 'generic-subgraph-generator',
            code: lambda.Code.file('./generic-subgraph-generator.zip'),
        });
        generic_subgraph_generator.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addAction('s3:ActionList')
            .addResource(raw_logs_bucket.bucketArn));
        raw_logs_bucket.grantRead(generic_subgraph_generator.role);
        let generic_subgraph_generator_queue = new sqs.Queue(this, 'generic-subgraph-generator-queue');
        subscribe_lambda_to_queue(this, 'genericSubgraphGenerator', generic_subgraph_generator, generic_subgraph_generator_queue);
        // Add retry handler for node identifier retry handler
        let node_identifier_rth_dead_letter_queue = new sqs.Queue(this, 'node-identifier-rth-dead-letter-queue');
        let node_identifier_retry_handler = new lambda.Function(this, 'node-identifier-retry-handler', {
            runtime: lambda.Runtime.Go1x,
            handler: 'node-identifier-retry-handler',
            code: lambda.Code.file('./node-identifier-retry-handler.zip'),
            vpc: history_db_vpc
        });
        node_identifier_retry_handler.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addAction('s3:ActionList')
            .addResource(unid_subgraphs_generated_bucket.bucketArn));
        unid_subgraphs_generated_bucket.grantRead(node_identifier_retry_handler.role);
        let node_identifier_retry_handler_queue = new sqs.Queue(this, 'node-identifier-retry-handler-queue', {
            deadLetterQueue: { queue: node_identifier_rth_dead_letter_queue, maxReceiveCount: 5 }
        });
        subscribe_lambda_to_queue(this, 'nodeIdentifierRetryHandler', node_identifier_retry_handler, node_identifier_retry_handler_queue);
        // Node Identifier
        let node_identifier = new lambda.Function(this, 'node-identifier', {
            runtime: lambda.Runtime.Go1x,
            handler: 'node-identifier',
            code: lambda.Code.file('./node-identifier.zip'),
            vpc: history_db_vpc
        });
        node_identifier.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addAction('s3:ActionList')
            .addResource(unid_subgraphs_generated_bucket.bucketArn));
        unid_subgraphs_generated_bucket.grantRead(node_identifier.role);
        let node_identifier_queue = new sqs.Queue(this, 'node-identifier-queue', {
            deadLetterQueue: { queue: node_identifier_retry_handler_queue, maxReceiveCount: 5 }
        });
        subscribe_lambda_to_queue(this, 'nodeIdentifier', node_identifier, node_identifier_queue);
        // Graph Merge Service
        let graph_merger = new lambda.Function(this, 'graph-merger', {
            runtime: lambda.Runtime.Go1x,
            handler: 'graph-merger',
            code: lambda.Code.file('./graph-merger.zip'),
            vpc: dgraph_vpc
        });
        let graph_merger_queue = new sqs.Queue(this, 'graph-merger-queue');
        subscribe_lambda_to_queue(this, 'graphMerger', graph_merger, graph_merger_queue);
        graph_merger.addToRolePolicy(new cdk.PolicyStatement()
            .addAction('s3:GetObject')
            .addAction('s3:ActionGetBucket')
            .addAction('s3:ActionList')
            .addResource(subgraphs_generated_bucket.bucketArn));
        subgraphs_generated_bucket.grantRead(graph_merger.role);
        // Python Malicious Word Macro Execution
        let word_macro_analyzer = new lambda.Function(this, 'word-macro-analyzer', {
            runtime: lambda.Runtime.Python36,
            handler: 'word-macro-analyzer.lambda_handler',
            code: lambda.Code.file('./word-macro-analyzer.zip'),
            vpc: dgraph_vpc
        });
        let word_macro_analyzer_queue = new sqs.Queue(this, 'word-macro-analyzer-queue');
        subscribe_lambda_to_queue(this, 'wordMacroAnalyzer', word_macro_analyzer, word_macro_analyzer_queue);
        subgraph_merged_topic.subscribeQueue(word_macro_analyzer_queue);
        let engagement_creation_service = new lambda.Function(this, 'engagement-creation-service', {
            runtime: lambda.Runtime.Go1x,
            handler: 'engagement-creation-service',
            code: lambda.Code.file('./engagement-creation-service.zip'),
            vpc: dgraph_vpc
        });
        let engagement_creation_service_queue = new sqs.Queue(this, 'engagement-creation-service-queue');
        subscribe_lambda_to_queue(this, 'engagementCreator', engagement_creation_service, engagement_creation_service_queue);
        // SNS -> Service Queue
        raw_logs_topic.subscribeQueue(generic_subgraph_generator_queue);
        unid_subgraphs_generated_topic.subscribeQueue(node_identifier_queue);
        subgraphs_generated_topic.subscribeQueue(graph_merger_queue);
        // incident_topic.subscribeQueue(engagement_creation_service_queue);
        // Service -> S3
        unid_subgraphs_generated_bucket.grantWrite(generic_subgraph_generator.role);
        subgraphs_generated_bucket.grantWrite(node_identifier.role);
        subgraphs_generated_bucket.grantWrite(node_identifier_retry_handler.role);
        // Service -> SNS
        subgraph_merged_topic.grantPublish(graph_merger.role);
        incident_topic.grantPublish(word_macro_analyzer.role);
    }
}
class Grapl extends cdk.App {
    constructor(argv) {
        super(argv);
        new GraplStack(this, 'grapl');
    }
}
process.stdout.write(new Grapl(process.argv).run());
