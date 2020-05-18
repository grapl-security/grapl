"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const aws_lambda_event_sources_1 = require("@aws-cdk/aws-lambda-event-sources");
const aws_ec2_1 = require("@aws-cdk/aws-ec2");
const aws_lambda_1 = require("@aws-cdk/aws-lambda");
const core_1 = require("@aws-cdk/core");
const aws_ecs_1 = require("@aws-cdk/aws-ecs");
const AWS = require('aws-sdk');
const uuidv4 = require('uuid/v4');
const sagemaker = require("@aws-cdk/aws-sagemaker");
const cloudmap = require("@aws-cdk/aws-servicediscovery");
const s3Subs = require("@aws-cdk/aws-s3-notifications");
const s3deploy = require("@aws-cdk/aws-s3-deployment");
const snsSubs = require("@aws-cdk/aws-sns-subscriptions");
const elasticache = require("@aws-cdk/aws-elasticache");
const cdk = require("@aws-cdk/core");
const s3 = require("@aws-cdk/aws-s3");
const sns = require("@aws-cdk/aws-sns");
const sqs = require("@aws-cdk/aws-sqs");
const ec2 = require("@aws-cdk/aws-ec2");
const servicediscovery = require("@aws-cdk/aws-servicediscovery");
const lambda = require("@aws-cdk/aws-lambda");
const iam = require("@aws-cdk/aws-iam");
const dynamodb = require("@aws-cdk/aws-dynamodb");
const ecs = require("@aws-cdk/aws-ecs");
const apigateway = require("@aws-cdk/aws-apigateway");
const env = require('node-env-file');
const dir = require('node-dir');
class RedisCluster extends cdk.Construct {
    constructor(scope, id, vpc) {
        super(scope, id);
        // Define a group for telling Elasticache which subnets to put cache nodes in.
        const subnetGroup = new elasticache.CfnSubnetGroup(this, `${id}-subnet-group`, {
            description: `List of subnets used for redis cache ${id}`,
            subnetIds: vpc.privateSubnets.map(function (subnet) {
                return subnet.subnetId;
            }),
            cacheSubnetGroupName: id + 'SubnetGroupName'
        });
        // The security group that defines network level access to the cluster
        this.securityGroup = new ec2.SecurityGroup(this, `${id}-security-group`, { vpc: vpc });
        this.connections = new ec2.Connections({
            securityGroups: [this.securityGroup],
            defaultPort: ec2.Port.tcp(6379)
        });
        this.connections.allowFromAnyIpv4(aws_ec2_1.Port.tcp(6379));
        // The cluster resource itself.
        this.cluster = new elasticache.CfnCacheCluster(this, `${id}-cluster`, {
            cacheNodeType: 'cache.t2.small',
            engine: 'redis',
            numCacheNodes: 1,
            autoMinorVersionUpgrade: true,
            cacheSubnetGroupName: subnetGroup.cacheSubnetGroupName,
            vpcSecurityGroupIds: [
                this.securityGroup.securityGroupId
            ]
        });
    }
}
class Queues {
    constructor(stack, queue_name) {
        this.dead_letter_queue = new sqs.Queue(stack, queue_name + '-dead-letter');
        this.retry_queue = new sqs.Queue(stack, queue_name + '-retry', {
            deadLetterQueue: { queue: this.dead_letter_queue, maxReceiveCount: 10 },
            visibilityTimeout: core_1.Duration.seconds(360)
        });
        this.queue = new sqs.Queue(stack, queue_name, {
            deadLetterQueue: { queue: this.retry_queue, maxReceiveCount: 5 },
            visibilityTimeout: core_1.Duration.seconds(180)
        });
    }
}
class UserAuthDb extends cdk.Stack {
    constructor(parent, id) {
        super(parent, id + '-stack');
        this.user_auth_table = new dynamodb.Table(this, 'user_auth_table', {
            tableName: "user_auth_table",
            partitionKey: {
                name: 'username',
                type: dynamodb.AttributeType.STRING
            },
            serverSideEncryption: true,
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
    }
    allowRead(service) {
        this.user_auth_table.grantReadData(service.event_handler.role);
        this.user_auth_table.grantReadData(service.event_retry_handler.role);
    }
    allowReadWrite(service) {
        this.user_auth_table.grantReadData(service.event_handler.role);
        this.user_auth_table.grantReadData(service.event_retry_handler.role);
    }
    allowReadFromRole(role) {
        this.user_auth_table.grantReadData(role);
    }
    allowReadWriteFromRole(role) {
        this.user_auth_table.grantReadWriteData(role);
    }
}
class EngagementEdge extends cdk.Stack {
    constructor(parent, name, hostname, jwt_secret, engagement_graph, user_auth_table, vpc) {
        super(parent, name + '-stack');
        this.name = name + process.env.BUCKET_PREFIX;
        this.integrationName = name + process.env.BUCKET_PREFIX + 'Integration';
        this.event_handler = new lambda.Function(this, name, {
            runtime: aws_lambda_1.Runtime.PYTHON_3_7,
            handler: `engagement_edge.app`,
            code: lambda.Code.fromAsset(`./engagement_edge.zip`),
            vpc: vpc,
            environment: {
                "EG_ALPHAS": engagement_graph.alphaNames.join(","),
                "JWT_SECRET": jwt_secret,
                "USER_AUTH_TABLE": user_auth_table.user_auth_table.tableName,
                "BUCKET_PREFIX": process.env.BUCKET_PREFIX
            },
            timeout: core_1.Duration.seconds(25),
            memorySize: 256,
        });
        user_auth_table.allowReadFromRole(this.event_handler.role);
        this.integration = new apigateway.LambdaRestApi(this, this.integrationName, {
            handler: this.event_handler,
        });
    }
}
class Service {
    constructor(stack, name, environment, vpc, retry_code_name, opt) {
        let runtime = null;
        if (opt && opt.runtime) {
            runtime = opt.runtime;
        }
        else {
            runtime = { name: "provided", supportsInlineCode: true };
        }
        let handler = null;
        if (runtime === aws_lambda_1.Runtime.PYTHON_3_7) {
            handler = `${name}.lambda_handler`;
        }
        else {
            handler = name;
        }
        const queues = new Queues(stack, name + '-queue');
        if (environment) {
            environment.QUEUE_URL = queues.queue.queueUrl;
            environment.RUST_BACKTRACE = "1";
        }
        let event_handler = new lambda.Function(stack, name, {
            runtime: runtime,
            handler: handler,
            code: lambda.Code.asset(`./${name}.zip`),
            vpc: vpc,
            environment: {
                IS_RETRY: "False",
                ...environment
            },
            timeout: core_1.Duration.seconds(180),
            memorySize: 256,
        });
        if (!retry_code_name) {
            retry_code_name = name;
        }
        if (environment) {
            environment.QUEUE_URL = queues.retry_queue.queueUrl;
        }
        let event_retry_handler = new lambda.Function(stack, name + '-retry-handler', {
            runtime: runtime,
            handler: handler,
            code: lambda.Code.asset(`./${retry_code_name}.zip`),
            vpc: vpc,
            environment: {
                IS_RETRY: "True",
                ...environment
            },
            timeout: core_1.Duration.seconds(360),
            memorySize: 512,
        });
        event_handler.addEventSource(new aws_lambda_event_sources_1.SqsEventSource(queues.queue, { batchSize: 1 }));
        event_retry_handler.addEventSource(new aws_lambda_event_sources_1.SqsEventSource(queues.retry_queue, { batchSize: 1 }));
        queues.queue.grantConsumeMessages(event_handler);
        queues.retry_queue.grantConsumeMessages(event_retry_handler);
        this.queues = queues;
        this.event_handler = event_handler;
        this.event_retry_handler = event_retry_handler;
    }
    readsFrom(bucket, with_list) {
        let policy = new iam.PolicyStatement();
        policy.addActions('s3:GetObject', 's3:ActionGetBucket');
        if (with_list === true) {
            policy.addActions('s3:ListObjects');
        }
        policy.addResources(bucket.bucketArn);
        this.event_handler.addToRolePolicy(policy);
        this.event_retry_handler.addToRolePolicy(policy);
        // TODO: This is adding more permissions than necessary
        bucket.grantRead(this.event_handler.role);
        bucket.grantRead(this.event_retry_handler.role);
    }
    publishesToTopic(publishes_to) {
        const topicPolicy = new iam.PolicyStatement();
        topicPolicy.addActions('sns:CreateTopic');
        topicPolicy.addResources(publishes_to.topicArn);
        this.event_handler.addToRolePolicy(topicPolicy);
        this.event_retry_handler.addToRolePolicy(topicPolicy);
        publishes_to.grantPublish(this.event_handler.role);
        publishes_to.grantPublish(this.event_retry_handler.role);
    }
    publishesToBucket(publishes_to) {
        publishes_to.grantWrite(this.event_handler.role);
        publishes_to.grantWrite(this.event_retry_handler.role);
    }
}
class SessionIdentityCache extends cdk.Stack {
    constructor(parent, vpc) {
        super(parent, 'session-identity-cache-stack');
        // const zone = new route53.PrivateHostedZone(this, 'HostedZone', {
        //     zoneName: 'sessionid.cache',
        //     vpc_props
        // });
    }
}
class EventEmitter {
    constructor(stack, eventName) {
        this.bucket =
            new s3.Bucket(stack, eventName + '-bucket', {
                bucketName: process.env.BUCKET_PREFIX + eventName + "-bucket"
            });
        // SNS Topics
        this.topic =
            new sns.Topic(stack, `${eventName}-topic`, {
                topicName: `${eventName}-topic`
            });
        this.bucket
            .addEventNotification(s3.EventType.OBJECT_CREATED, new s3Subs.SnsDestination(this.topic));
    }
}
class EventEmitters extends cdk.Stack {
    constructor(parent, id) {
        super(parent, id + '-stack');
        let raw_logs_bucket = new s3.Bucket(this, id + '-raw-log-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-raw-log-bucket"
        });
        let sysmon_logs_bucket = new s3.Bucket(this, id + '-sysmon-log-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-sysmon-log-bucket"
        });
        let identity_mappings_bucket = new s3.Bucket(this, id + '-identity-mappings-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-identity-mappings-bucket"
        });
        let unid_subgraphs_generated_bucket = new s3.Bucket(this, id + '-unid-subgraphs-generated-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-unid-subgraphs-generated-bucket"
        });
        let subgraphs_generated_bucket = new s3.Bucket(this, id + '-subgraphs-generated-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-subgraphs-generated-bucket"
        });
        let analyzers_bucket = new s3.Bucket(this, id + '-analyzers-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-analyzers-bucket"
        });
        let model_plugins_bucket = new s3.Bucket(this, id + '-model-plugins-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-model-plugins-bucket"
        });
        let subgraphs_merged_bucket = new s3.Bucket(this, id + '-subgraphs-merged-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-subgraphs-merged-bucket"
        });
        let dispatched_analyzer_bucket = new s3.Bucket(this, id + '-dispatched-analyzer-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-dispatched-analyzer-bucket"
        });
        let analyzer_matched_subgraphs_bucket = new s3.Bucket(this, id + '-analyzer-matched-subgraphs-bucket', {
            bucketName: process.env.BUCKET_PREFIX + "-analyzer-matched-subgraphs-bucket"
        });
        // SNS Topics
        let incident_topic = new sns.Topic(this, id + '-incident-topic', {
            topicName: 'incident-topic'
        });
        let raw_logs_topic = new sns.Topic(this, id + '-raw-log-topic', {
            topicName: 'raw-log-topic'
        });
        let sysmon_logs_topic = new sns.Topic(this, id + '-sysmon-log-topic', {
            topicName: 'sysmon-log-topic'
        });
        let identity_mappings_topic = new sns.Topic(this, id + '-identity-mappings-topic', {
            topicName: 'identity-mappings-topic'
        });
        let unid_subgraphs_generated_topic = new sns.Topic(this, id + '-unid-subgraphs-generated-topic', {
            topicName: 'unid-subgraphs-generated-topic'
        });
        let subgraphs_generated_topic = new sns.Topic(this, id + '-subgraphs-generated-topic', {
            topicName: 'subgraphs-generated-topic'
        });
        let subgraph_merged_topic = new sns.Topic(this, id + '-subgraphs-merged-topic', {
            topicName: 'subgraphs-merged-topic'
        });
        let dispatched_analyzer_topic = new sns.Topic(this, id + '-dispatched-analyzer-topic', {
            topicName: 'dispatched-analyzer-topic'
        });
        let analyzer_matched_subgraphs_topic = new sns.Topic(this, id + '-analyzer-matched-subgraphs-topic', {
            topicName: 'analyzer-matched-subgraphs-topic'
        });
        let engagements_created_topic = new sns.Topic(this, id + '-engagements-created-topic', {
            topicName: 'engagements-created-topic'
        });
        // S3 -> SNS Events
        raw_logs_bucket
            .addEventNotification(s3.EventType.OBJECT_CREATED, new s3Subs.SnsDestination(raw_logs_topic));
        sysmon_logs_bucket
            .addEventNotification(s3.EventType.OBJECT_CREATED, new s3Subs.SnsDestination(sysmon_logs_topic));
        identity_mappings_bucket
            .addEventNotification(s3.EventType.OBJECT_CREATED, new s3Subs.SnsDestination(identity_mappings_topic));
        unid_subgraphs_generated_bucket
            .addEventNotification(s3.EventType.OBJECT_CREATED, new s3Subs.SnsDestination(unid_subgraphs_generated_topic));
        subgraphs_generated_bucket
            .addEventNotification(s3.EventType.OBJECT_CREATED, new s3Subs.SnsDestination(subgraphs_generated_topic));
        subgraphs_merged_bucket
            .addEventNotification(s3.EventType.OBJECT_CREATED, new s3Subs.SnsDestination(subgraph_merged_topic));
        dispatched_analyzer_bucket
            .addEventNotification(s3.EventType.OBJECT_CREATED, new s3Subs.SnsDestination(dispatched_analyzer_topic));
        analyzer_matched_subgraphs_bucket
            .addEventNotification(s3.EventType.OBJECT_CREATED, new s3Subs.SnsDestination(analyzer_matched_subgraphs_topic));
        this.raw_logs_bucket = raw_logs_bucket;
        this.sysmon_logs_bucket = sysmon_logs_bucket;
        this.identity_mappings_bucket = identity_mappings_bucket;
        this.unid_subgraphs_generated_bucket = unid_subgraphs_generated_bucket;
        this.subgraphs_generated_bucket = subgraphs_generated_bucket;
        this.analyzers_bucket = analyzers_bucket;
        this.model_plugins_bucket = model_plugins_bucket;
        this.subgraphs_merged_bucket = subgraphs_merged_bucket;
        this.dispatched_analyzer_bucket = dispatched_analyzer_bucket;
        this.analyzer_matched_subgraphs_bucket = analyzer_matched_subgraphs_bucket;
        this.incident_topic = incident_topic;
        this.raw_logs_topic = raw_logs_topic;
        this.sysmon_logs_topic = sysmon_logs_topic;
        this.identity_mappings_topic = identity_mappings_topic;
        this.unid_subgraphs_generated_topic = unid_subgraphs_generated_topic;
        this.subgraphs_generated_topic = subgraphs_generated_topic;
        this.subgraph_merged_topic = subgraph_merged_topic;
        this.dispatched_analyzer_topic = dispatched_analyzer_topic;
        this.analyzer_matched_subgraphs_topic = analyzer_matched_subgraphs_topic;
        this.engagements_created_topic = engagements_created_topic;
    }
}
class SysmonSubgraphGenerator extends cdk.Stack {
    constructor(parent, id, reads_from, subscribes_to, writes_to, vpc) {
        super(parent, id + '-stack');
        const sysmon_event_cache = new RedisCluster(this, id + 'sysmoneventcache', vpc);
        sysmon_event_cache.connections.allowFromAnyIpv4(aws_ec2_1.Port.allTcp());
        const environment = {
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "EVENT_CACHE_ADDR": sysmon_event_cache.cluster.attrRedisEndpointAddress,
            "EVENT_CACHE_PORT": sysmon_event_cache.cluster.attrRedisEndpointPort,
        };
        const service = new Service(this, 'sysmon-subgraph-generator', environment, vpc);
        service.event_handler.connections.allowToAnyIpv4(aws_ec2_1.Port.allTraffic());
        service.event_retry_handler.connections.allowToAnyIpv4(aws_ec2_1.Port.allTraffic());
        service.readsFrom(reads_from);
        addSubscription(this, subscribes_to, new snsSubs.SqsSubscription(service.queues.queue), true);
        service.publishesToBucket(writes_to);
    }
}
class GenericSubgraphGenerator extends cdk.Stack {
    constructor(parent, id, reads_from, subscribes_to, writes_to, vpc) {
        super(parent, id + '-stack');
        const generic_event_cache = new RedisCluster(this, id + 'genericeventcache', vpc);
        generic_event_cache.connections.allowFromAnyIpv4(aws_ec2_1.Port.allTcp());
        const environment = {
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "GENERIC_EVENT_CACHE_ADDR": generic_event_cache.cluster.attrRedisEndpointAddress,
            "GENERIC_EVENT_CACHE_PORT": generic_event_cache.cluster.attrRedisEndpointPort,
        };
        const service = new Service(this, 'generic-subgraph-generator', environment, vpc);
        service.readsFrom(reads_from);
        service.event_handler.connections.allowToAnyIpv4(aws_ec2_1.Port.tcp(parseInt(generic_event_cache.cluster.attrRedisEndpointPort)));
        service.event_retry_handler.connections.allowToAnyIpv4(aws_ec2_1.Port.tcp(parseInt(generic_event_cache.cluster.attrRedisEndpointPort)));
        addSubscription(this, subscribes_to, new snsSubs.SqsSubscription(service.queues.queue), true);
        service.publishesToBucket(writes_to);
    }
}
function addSubscription(scope, topic, subscription, raw) {
    const config = subscription.bind(topic);
    new sns.Subscription(scope, 'Subscription', {
        topic: topic,
        endpoint: config.endpoint,
        filterPolicy: config.filterPolicy,
        protocol: config.protocol,
        rawMessageDelivery: raw || config.rawMessageDelivery
    });
}
class NodeIdentifier extends cdk.Stack {
    constructor(parent, id, reads_from, subscribes_to, writes_to, history_db, vpc) {
        super(parent, id + '-stack');
        const retry_identity_cache = new RedisCluster(this, id + 'retrycache', vpc);
        retry_identity_cache.connections.allowFromAnyIpv4(aws_ec2_1.Port.allTcp());
        const environment = {
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "RETRY_IDENTITY_CACHE_ADDR": retry_identity_cache.cluster.attrRedisEndpointAddress,
            "RETRY_IDENTITY_CACHE_PORT": retry_identity_cache.cluster.attrRedisEndpointPort,
        };
        const service = new Service(this, 'node-identifier', environment, vpc, 'node-identifier-retry-handler');
        service.readsFrom(reads_from);
        history_db.allowReadWrite(service);
        service.publishesToBucket(writes_to);
        addSubscription(this, subscribes_to, new snsSubs.SqsSubscription(service.queues.queue), true);
        service.event_handler.connections.allowToAnyIpv4(aws_ec2_1.Port.tcp(parseInt(retry_identity_cache.cluster.attrRedisEndpointPort)));
        service.event_retry_handler.connections.allowToAnyIpv4(aws_ec2_1.Port.tcp(parseInt(retry_identity_cache.cluster.attrRedisEndpointPort)));
        service.event_handler.connections.allowToAnyIpv4(ec2.Port.tcp(443), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.tcp(443), 'Allow outbound to S3');
    }
}
class GraphMerger extends cdk.Stack {
    constructor(parent, id, reads_from, subscribes_to, writes_to, master_graph, vpc) {
        super(parent, id + '-stack');
        const graph_merge_cache = new RedisCluster(this, id + 'mergedcache', vpc);
        graph_merge_cache.connections.allowFromAnyIpv4(aws_ec2_1.Port.allTcp());
        graph_merge_cache.connections.allowFromAnyIpv4(aws_ec2_1.Port.allTcp());
        const environment = {
            "SUBGRAPH_MERGED_BUCKET": writes_to.bucketName,
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "MG_ALPHAS": master_graph.alphaNames.join(","),
            "MERGED_CACHE_ADDR": graph_merge_cache.cluster.attrRedisEndpointAddress,
            "MERGED_CACHE_PORT": graph_merge_cache.cluster.attrRedisEndpointPort,
        };
        const service = new Service(this, 'graph-merger', environment, vpc);
        service.readsFrom(reads_from);
        service.publishesToBucket(writes_to);
        addSubscription(this, subscribes_to, new snsSubs.SqsSubscription(service.queues.queue), true);
        //
        // service.event_handler.connections
        //     .allowToAnyIpv4(new ec2.Port({
        //
        //     }), 'Allow outbound to S3');
        // service.event_retry_handler.connections
        //     .allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
    }
}
class AnalyzerDispatch extends cdk.Stack {
    constructor(parent, id, subscribes_to, // The SNS Topic that we must subscribe to our queue
    writes_to, reads_from, analyzer_bucket, vpc) {
        super(parent, id + '-stack');
        const dispatch_event_cache = new RedisCluster(this, id + 'dispatcheventcache', vpc);
        dispatch_event_cache.connections.allowFromAnyIpv4(aws_ec2_1.Port.allTcp());
        const environment = {
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "EVENT_CACHE_ADDR": dispatch_event_cache.cluster.attrRedisEndpointAddress,
            "EVENT_CACHE_PORT": dispatch_event_cache.cluster.attrRedisEndpointPort,
            "DISPATCHED_ANALYZER_BUCKET": writes_to.bucketName,
            "SUBGRAPH_MERGED_BUCKET": reads_from.bucketName,
        };
        const service = new Service(this, 'analyzer-dispatcher', environment, vpc);
        service.publishesToBucket(writes_to);
        // We need the List capability to find each of the analyzers
        service.readsFrom(analyzer_bucket, true);
        service.readsFrom(reads_from);
        addSubscription(this, subscribes_to, new snsSubs.SqsSubscription(service.queues.queue), true);
        service.event_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
    }
}
class AnalyzerExecutor extends cdk.Stack {
    constructor(parent, id, subscribes_to, reads_analyzers_from, reads_events_from, writes_events_to, model_plugins_bucket, master_graph, vpc) {
        super(parent, id + '-stack');
        this.count_cache = new RedisCluster(this, id + 'countcache', vpc);
        this.hit_cache = new RedisCluster(this, id + 'hitcache', vpc);
        this.message_cache = new RedisCluster(this, id + 'msgcache', vpc);
        const environment = {
            "ANALYZER_MATCH_BUCKET": writes_events_to.bucketName,
            "BUCKET_PREFIX": process.env.BUCKET_PREFIX,
            "MG_ALPHAS": master_graph.alphaNames.join(","),
            "COUNTCACHE_ADDR": this.count_cache.cluster.attrRedisEndpointAddress,
            "COUNTCACHE_PORT": this.count_cache.cluster.attrRedisEndpointPort,
            "MESSAGECACHE_ADDR": this.message_cache.cluster.attrRedisEndpointAddress,
            "MESSAGECACHE_PORT": this.message_cache.cluster.attrRedisEndpointPort,
            "HITCACHE_ADDR": this.hit_cache.cluster.attrRedisEndpointAddress,
            "HITCACHE_PORT": this.hit_cache.cluster.attrRedisEndpointPort,
            "GRPC_ENABLE_FORK_SUPPORT": "1",
        };
        const service = new Service(this, 'analyzer-executor', environment, vpc, null, {
            runtime: aws_lambda_1.Runtime.PYTHON_3_7
        });
        this.count_cache.connections.allowFromAnyIpv4(aws_ec2_1.Port.tcp(6379));
        this.hit_cache.connections.allowFromAnyIpv4(aws_ec2_1.Port.tcp(6379));
        this.message_cache.connections.allowFromAnyIpv4(aws_ec2_1.Port.tcp(6379));
        service.publishesToBucket(writes_events_to);
        // We need the List capability to find each of the analyzers
        service.readsFrom(reads_analyzers_from, true);
        service.readsFrom(model_plugins_bucket, true);
        service.readsFrom(reads_events_from);
        // Need to be able to GetObject in order to HEAD, can be replaced with
        // a cache later, but safe so long as there is no LIST
        let policy = new iam.PolicyStatement();
        policy.addActions('s3:GetObject');
        policy.addResources(writes_events_to.bucketArn);
        service.event_handler.addToRolePolicy(policy);
        service.event_retry_handler.addToRolePolicy(policy);
        addSubscription(this, subscribes_to, new snsSubs.SqsSubscription(service.queues.queue), true);
        service.event_handler.connections.allowToAnyIpv4(ec2.Port.allTraffic(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.allTraffic(), 'Allow outbound to S3');
    }
}
class EngagementCreator extends cdk.Stack {
    constructor(parent, id, reads_from, subscribes_to, publishes_to, master_graph, engagement_graph, vpc) {
        super(parent, id + '-stack');
        const environment = {
            "MG_ALPHAS": master_graph.alphaNames.join(","),
            "EG_ALPHAS": engagement_graph.alphaNames.join(","),
        };
        const service = new Service(this, 'engagement-creator', environment, vpc, null, {
            runtime: aws_lambda_1.Runtime.PYTHON_3_7
        });
        service.readsFrom(reads_from);
        service.publishesToTopic(publishes_to);
        addSubscription(this, subscribes_to, new snsSubs.SqsSubscription(service.queues.queue), true);
        service.event_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
    }
}
class Networks extends cdk.Stack {
    constructor(parent, id) {
        super(parent, id + '-stack');
        this.grapl_vpc = new ec2.Vpc(this, 'GraplVPC', {
            natGateways: 1,
            enableDnsHostnames: true,
            enableDnsSupport: true,
        });
    }
}
class Zero {
    constructor(parent, stack, graph, id, cluster, peer, idx) {
        const zeroTask = new ecs.Ec2TaskDefinition(stack, id, {
            networkMode: aws_ecs_1.NetworkMode.AWS_VPC,
        });
        let command = ["dgraph", "zero", `--my=${id}.${graph}.grapl:5080`,
            "--replicas=3",
            `--idx=${idx}`,
            "--alsologtostderr"];
        if (peer) {
            command.push(`--peer=${peer}.${graph}.grapl:5080`);
        }
        const logDriver = new ecs.AwsLogDriver({
            streamPrefix: `ecs${graph + id}`,
        });
        zeroTask.addContainer(id + 'Container', {
            // --my is our own hostname (graph + id)
            // --peer is the other dgraph zero hostname
            image: ecs.ContainerImage.fromRegistry("dgraph/dgraph"),
            command,
            logging: logDriver,
            memoryReservationMiB: 1024,
        });
        const zeroService = new ecs.Ec2Service(stack, id + 'Service', {
            cluster,
            taskDefinition: zeroTask,
            cloudMapOptions: {
                name: id,
                dnsRecordType: servicediscovery.DnsRecordType.A,
                dnsTtl: core_1.Duration.seconds(300),
            }
        });
        this.name = `${id}.${graph}.grapl`;
        zeroService.connections.allowFromAnyIpv4(ec2.Port.allTcp());
    }
}
class Alpha {
    constructor(parent, stack, graph, id, cluster, zero) {
        const alphaTask = new ecs.Ec2TaskDefinition(stack, id, {
            networkMode: aws_ecs_1.NetworkMode.AWS_VPC,
        });
        const logDriver = new ecs.AwsLogDriver({
            streamPrefix: `ecs${graph + id}`,
        });
        alphaTask.addContainer(id + graph + 'Container', {
            image: ecs.ContainerImage.fromRegistry("dgraph/dgraph"),
            command: [
                "dgraph", "alpha", `--my=${id}.${graph}.grapl:7080`,
                "--lru_mb=1024", `--zero=${zero}.${graph}.grapl:5080`,
                "--alsologtostderr"
            ],
            logging: logDriver,
            memoryReservationMiB: 2048,
        });
        const alphaService = new ecs.Ec2Service(stack, id + 'Service', {
            cluster,
            taskDefinition: alphaTask,
            cloudMapOptions: {
                name: id,
                dnsRecordType: servicediscovery.DnsRecordType.A,
                dnsTtl: core_1.Duration.seconds(300),
            }
        });
        this.name = `${id}.${graph}.grapl`;
        alphaService.connections.allowFromAnyIpv4(ec2.Port.allTcp());
    }
}
class DGraphEcs extends cdk.Stack {
    constructor(parent, id, vpc, zeroCount, alphaCount) {
        super(parent, id + '-stack');
        const cluster = new ecs.Cluster(this, id + '-EcsCluster', {
            vpc: vpc
        });
        cluster.connections.allowInternally(aws_ec2_1.Port.allTcp());
        this.cluster = cluster;
        const namespace = cluster.addDefaultCloudMapNamespace({
            name: id + '.grapl',
            type: cloudmap.NamespaceType.DNS_PRIVATE,
            vpc
        });
        cluster.addCapacity(id + "ZeroGroupCapacity", {
            instanceType: new ec2.InstanceType("t3a.small"),
            minCapacity: zeroCount,
            desiredCapacity: zeroCount,
            maxCapacity: zeroCount,
        });
        const zero0 = new Zero(parent, this, id, 'zero0', cluster, null, 1);
        for (let i = 1; i < zeroCount; i++) {
            new Zero(parent, this, id, `zero${i}`, cluster, 'zero0', 1 + i);
        }
        this.alphaNames = [];
        cluster.addCapacity(id + "AlphaGroupCapacity", {
            instanceType: new ec2.InstanceType("t3a.2xlarge"),
            minCapacity: alphaCount,
            desiredCapacity: alphaCount,
            maxCapacity: alphaCount,
        });
        for (let i = 0; i < alphaCount; i++) {
            const alpha = new Alpha(parent, this, id, `alpha${i}`, // increment for each alpha
            cluster, "zero0");
            this.alphaNames.push(alpha.name);
        }
    }
}
class EngagementNotebook extends cdk.Stack {
    constructor(parent, id, user_auth_db, vpc) {
        super(parent, id + '-notebook-stack');
        this.securityGroup = new ec2.SecurityGroup(this, `${id}-notebook-security-group`, { vpc: vpc });
        this.connections = new ec2.Connections({
            securityGroups: [this.securityGroup],
            defaultPort: ec2.Port.allTcp()
        });
        const role = new iam.Role(this, id + 'notebook-role', {
            assumedBy: new iam.ServicePrincipal('sagemaker.amazonaws.com')
        });
        user_auth_db.allowReadWriteFromRole(role);
        const _notebook = new sagemaker.CfnNotebookInstance(this, id + '-sagemaker-endpoint', {
            instanceType: 'ml.t2.medium',
            securityGroupIds: [this.securityGroup.securityGroupId],
            subnetId: vpc.privateSubnets[0].subnetId,
            directInternetAccess: 'Enabled',
            roleArn: role.roleArn
        });
    }
}
const fs = require('fs'), path = require('path');
const replaceInFile = (toModify, toReplace, replaceWith, outputFile) => {
    return fs.readFile(toModify, (err, data) => {
        if (err) {
            return console.log(err);
        }
        const replaced = data
            .split(toReplace)
            .join(replaceWith)
            .split("const isLocal = true;")
            .join("const isLocal = false;");
        if (outputFile) {
            fs.writeFile(outputFile, replaced, 'utf8', (err) => {
                if (err)
                    return console.log(err);
            });
        }
        else {
            fs.writeFile(toModify, replaced, 'utf8', (err) => {
                if (err)
                    return console.log(err);
            });
        }
    }, 'utf8');
};
const getEdgeGatewayId = (integrationName, cb) => {
    let apigateway = new AWS.APIGateway();
    apigateway.getRestApis({}, function (err, data) {
        if (err) {
            console.log('Error getting edge gateway ID', err);
        }
        for (const item of data.items) {
            if (item.name === integrationName) {
                console.log(`restApi ID ${item.id}`);
                cb(item.id);
                return;
            }
        }
        console.warn(false, 'Could not find any integrations. Ensure you have deployed engagement edge.');
    });
};
class EngagementUx extends cdk.Stack {
    constructor(parent, id, edge) {
        super(parent, id + '-stack');
        const bucketName = process.env.BUCKET_PREFIX + id + '-bucket';
        const edgeBucket = new s3.Bucket(this, bucketName, {
            bucketName,
            publicReadAccess: true,
            websiteIndexDocument: 'index.html',
        });
        if (!fs.existsSync(path.join(__dirname, 'edge_ux_package/'))) {
            fs.mkdirSync(path.join(__dirname, 'edge_ux_package/'));
        }
        getEdgeGatewayId(edge.name + 'Integration', (gatewayId) => {
            const edgeUrl = `https://${gatewayId}.execute-api.${AWS.config.region}.amazonaws.com/prod/`;
            const toReplace = "__engagement_ux_standin__hostname__";
            const replacement = `${edgeUrl}`;
            console.log(__dirname);
            dir.readFiles(path.join(__dirname, 'edge_ux/'), function (err, content, filename, next) {
                if (err)
                    throw err;
                const targetDir = path.dirname(filename).replace("edge_ux", "edge_ux_package");
                if (!fs.existsSync(targetDir)) {
                    fs.mkdirSync(targetDir, { recursive: true });
                }
                const newPath = filename.replace("edge_ux", "edge_ux_package");
                replaceInFile(filename, toReplace, replacement, newPath);
                next();
            }, function (err, files) {
                if (err)
                    throw err;
            });
        });
        new s3deploy.BucketDeployment(this, id + 'Ux', {
            sources: [s3deploy.Source.asset('./edge_ux_package')],
            destinationBucket: edgeBucket,
        });
    }
}
class HistoryDb extends cdk.Stack {
    constructor(parent, id) {
        super(parent, id + '-stack');
        this.proc_history = new dynamodb.Table(this, 'process_history_table', {
            tableName: "process_history_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'create_time',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
        this.file_history = new dynamodb.Table(this, 'file_history_table', {
            tableName: "file_history_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'create_time',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
        this.outbound_connection_history = new dynamodb.Table(this, 'outbound_connection_history_table', {
            tableName: "outbound_connection_history_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'create_time',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
        this.inbound_connection_history = new dynamodb.Table(this, 'inbound_connection_history_table', {
            tableName: "inbound_connection_history_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'create_time',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
        this.network_connection_history = new dynamodb.Table(this, 'network_connection_history', {
            tableName: "network_connection_history_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'create_time',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
        this.ip_connection_history = new dynamodb.Table(this, 'ip_connection_history_table', {
            tableName: "ip_connection_history_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'create_time',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
        this.asset_history = new dynamodb.Table(this, 'asset_id_mappings', {
            tableName: "asset_id_mappings",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'c_timestamp',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
        this.dynamic_session_table = new dynamodb.Table(this, 'dynamic_session_table', {
            tableName: "dynamic_session_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            sortKey: {
                name: 'create_time',
                type: dynamodb.AttributeType.NUMBER
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
        this.static_mapping_table = new dynamodb.Table(this, 'static_mapping_table', {
            tableName: "static_mapping_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
        });
        this.node_id_retry_table = new dynamodb.Table(this, 'node_id_retry_table', {
            tableName: "node_id_retry_table",
            partitionKey: {
                name: 'pseudo_key',
                type: dynamodb.AttributeType.STRING
            },
            billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
            timeToLiveAttribute: "ttl_ts"
        });
    }
    allowReadWrite(service) {
        this.proc_history.grantReadWriteData(service.event_handler.role);
        this.file_history.grantReadWriteData(service.event_handler.role);
        this.outbound_connection_history.grantReadWriteData(service.event_handler.role);
        this.inbound_connection_history.grantReadWriteData(service.event_handler.role);
        this.network_connection_history.grantReadWriteData(service.event_handler.role);
        this.ip_connection_history.grantReadWriteData(service.event_handler.role);
        this.asset_history.grantReadWriteData(service.event_handler.role);
        this.node_id_retry_table.grantReadWriteData(service.event_handler.role);
        this.static_mapping_table.grantReadWriteData(service.event_handler.role);
        this.dynamic_session_table.grantReadWriteData(service.event_handler.role);
        this.proc_history.grantReadWriteData(service.event_retry_handler.role);
        this.file_history.grantReadWriteData(service.event_retry_handler.role);
        this.outbound_connection_history.grantReadWriteData(service.event_retry_handler.role);
        this.inbound_connection_history.grantReadWriteData(service.event_retry_handler.role);
        this.network_connection_history.grantReadWriteData(service.event_retry_handler.role);
        this.ip_connection_history.grantReadWriteData(service.event_retry_handler.role);
        this.asset_history.grantReadWriteData(service.event_retry_handler.role);
        this.node_id_retry_table.grantReadWriteData(service.event_retry_handler.role);
        this.static_mapping_table.grantReadWriteData(service.event_retry_handler.role);
        this.dynamic_session_table.grantReadWriteData(service.event_retry_handler.role);
    }
}
class Grapl extends cdk.App {
    constructor() {
        super();
        env(__dirname + '/.env');
        const mgZeroCount = Number(process.env.MG_ZEROS_COUNT) || 1;
        const mgAlphaCount = Number(process.env.MG_ALPHAS_COUNT) || 1;
        const egZeroCount = Number(process.env.EG_ZEROS_COUNT) || 1;
        const egAlphaCount = Number(process.env.EG_ALPHAS_COUNT) || 1;
        const jwtSecret = process.env.JWT_SECRET || uuidv4();
        let event_emitters = new EventEmitters(this, 'grapl-event-emitters');
        const network = new Networks(this, 'graplvpcs');
        const history_db = new HistoryDb(this, 'graplhistorydb');
        const master_graph = new DGraphEcs(this, 'mastergraphcluster', network.grapl_vpc, mgZeroCount, mgAlphaCount);
        const engagement_graph = new DGraphEcs(this, 'engagementgraphcluster', network.grapl_vpc, egZeroCount, egAlphaCount);
        new GenericSubgraphGenerator(this, 'grapl-generic-subgraph-generator', event_emitters.raw_logs_bucket, event_emitters.raw_logs_topic, event_emitters.unid_subgraphs_generated_bucket, network.grapl_vpc);
        new SysmonSubgraphGenerator(this, 'grapl-sysmon-subgraph-generator', event_emitters.sysmon_logs_bucket, event_emitters.sysmon_logs_topic, event_emitters.unid_subgraphs_generated_bucket, network.grapl_vpc);
        new NodeIdentifier(this, 'grapl-node-identifier', event_emitters.unid_subgraphs_generated_bucket, event_emitters.unid_subgraphs_generated_topic, event_emitters.subgraphs_generated_bucket, history_db, network.grapl_vpc);
        new GraphMerger(this, 'grapl-graph-merger', event_emitters.subgraphs_generated_bucket, event_emitters.subgraphs_generated_topic, event_emitters.subgraphs_merged_bucket, master_graph, network.grapl_vpc);
        new AnalyzerDispatch(this, 'grapl-analyzer-dispatcher', event_emitters.subgraph_merged_topic, event_emitters.dispatched_analyzer_bucket, event_emitters.subgraphs_merged_bucket, event_emitters.analyzers_bucket, network.grapl_vpc);
        new AnalyzerExecutor(this, 'grapl-analyzer-executor', event_emitters.dispatched_analyzer_topic, event_emitters.analyzers_bucket, event_emitters.dispatched_analyzer_bucket, event_emitters.analyzer_matched_subgraphs_bucket, master_graph, network.grapl_vpc);
        new EngagementCreator(this, 'grapl-engagement-creator', event_emitters.analyzer_matched_subgraphs_bucket, event_emitters.analyzer_matched_subgraphs_topic, event_emitters.engagements_created_topic, master_graph, engagement_graph, network.grapl_vpc);
        const user_auth_table = new UserAuthDb(this, 'grapl-user-auth-table');
        const engagement_edge = new EngagementEdge(this, 'engagementedge', 'engagementedge', jwtSecret, engagement_graph, user_auth_table, network.grapl_vpc);
        new EngagementNotebook(this, 'engagements', user_auth_table, network.grapl_vpc);
        new EngagementUx(this, 'engagement-ux', engagement_edge);
    }
}
new Grapl().synth();
