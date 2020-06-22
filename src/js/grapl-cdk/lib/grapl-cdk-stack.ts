import * as cdk from "@aws-cdk/core";
import * as s3 from "@aws-cdk/aws-s3";
import {BlockPublicAccess, BucketEncryption} from "@aws-cdk/aws-s3";
import * as sns from "@aws-cdk/aws-sns";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as events from "@aws-cdk/aws-events";
import * as targets from "@aws-cdk/aws-events-targets";
import * as lambda from "@aws-cdk/aws-lambda";
import {Runtime} from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as apigateway from "@aws-cdk/aws-apigateway";
import * as secretsmanager from '@aws-cdk/aws-secretsmanager';

import {Service} from "./service";
import {UserAuthDb} from "./userauthdb";
import {DGraphEcs} from "./dgraph";
import {HistoryDb} from "./historydb";
import {EventEmitter} from "./event_emitters";
import {RedisCluster} from "./redis";
import {EngagementNotebook} from "./engagement";

class SysmonSubgraphGenerator extends cdk.NestedStack {

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        vpc: ec2.IVpc,
        writes_to: s3.IBucket,
    ) {
        super(scope, id);

        const sysmon_log = new EventEmitter(this, prefix + '-sysmon-log');

        const event_cache = new RedisCluster(this, 'SysmonEventCache', vpc);
        event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(
            this,
            id,
            {
                environment: {
                    "BUCKET_PREFIX": prefix,
                    "EVENT_CACHE_ADDR": event_cache.cluster.attrRedisEndpointAddress,
                    "EVENT_CACHE_PORT": event_cache.cluster.attrRedisEndpointPort,
                },
                vpc: vpc,
                reads_from: sysmon_log.bucket,
                subscribes_to: sysmon_log.topic,
                writes_to: writes_to,
            });

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(
                parseInt(event_cache.cluster.attrRedisEndpointPort)
            ));

        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(
                parseInt(event_cache.cluster.attrRedisEndpointPort)
            ));

    }
}

class NodeIdentifier extends cdk.Construct {
    readonly bucket: s3.Bucket;
    readonly topic: sns.Topic;

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        vpc: ec2.IVpc,
        writes_to: s3.IBucket,
    ) {
        super(scope, id);

        const history_db = new HistoryDb(this, 'graplhistorydb');

        const unid_subgraphs = new EventEmitter(this, prefix + '-unid-subgraphs-generated');
        this.bucket = unid_subgraphs.bucket;
        this.topic = unid_subgraphs.topic;

        const retry_identity_cache = new RedisCluster(this, 'NodeIdentifierRetryCache', vpc);
        retry_identity_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(
            this,
            id,
            {
                environment: {
                    "BUCKET_PREFIX": prefix,
                    "RETRY_IDENTITY_CACHE_ADDR": retry_identity_cache.cluster.attrRedisEndpointAddress,
                    "RETRY_IDENTITY_CACHE_PORT": retry_identity_cache.cluster.attrRedisEndpointPort,
                },
                vpc: vpc,
                reads_from: unid_subgraphs.bucket,
                subscribes_to: unid_subgraphs.topic,
                writes_to: writes_to,
                retry_code_name: 'node-identifier-retry-handler',
            });

        history_db.allowReadWrite(service);

        service.event_handler.connections.allowToAnyIpv4(ec2.Port.tcp(
            parseInt(retry_identity_cache.cluster.attrRedisEndpointPort)
        ));

        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.tcp(
            parseInt(retry_identity_cache.cluster.attrRedisEndpointPort)
        ));

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(443), 'Allow outbound to S3'
        );
        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(443), 'Allow outbound to S3'
        );
    }
}

class GraphMerger extends cdk.NestedStack {
    readonly bucket: s3.Bucket;

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        vpc: ec2.IVpc,
        writes_to: s3.IBucket,
        master_graph: DGraphEcs,
    ) {
        super(scope, id);

        const subgraphs_generated = new EventEmitter(this, prefix + '-subgraphs-generated');
        this.bucket = subgraphs_generated.bucket;

        const graph_merge_cache = new RedisCluster(this, 'GraphMergerMergedCache', vpc);
        graph_merge_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(
            this,
            id,
            {
                environment: {
                    "BUCKET_PREFIX": prefix,
                    "SUBGRAPH_MERGED_BUCKET": writes_to.bucketName,
                    "MG_ALPHAS": master_graph.alphaNames.join(","),
                    "MERGED_CACHE_ADDR": graph_merge_cache.cluster.attrRedisEndpointAddress,
                    "MERGED_CACHE_PORT": graph_merge_cache.cluster.attrRedisEndpointPort,
                },
                vpc: vpc,
                reads_from: subgraphs_generated.bucket,
                subscribes_to: subgraphs_generated.topic,
                writes_to: writes_to,
            });
    }
}

class AnalyzerDispatch extends cdk.NestedStack {
    readonly bucket: s3.Bucket;
    readonly topic: sns.Topic;

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        vpc: ec2.IVpc,
        writes_to: s3.IBucket,
        analyzer_bucket: s3.IBucket,
    ) {
        super(scope, id);

        const subgraphs_merged = new EventEmitter(this, prefix + '-subgraphs-merged');
        this.bucket = subgraphs_merged.bucket;
        this.topic = subgraphs_merged.topic;

        const dispatch_event_cache = new RedisCluster(this, 'DispatchedEventCache', vpc);
        dispatch_event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(
            this,
            id,
            {
                environment: {
                    "BUCKET_PREFIX": prefix,
                    "EVENT_CACHE_ADDR": dispatch_event_cache.cluster.attrRedisEndpointAddress,
                    "EVENT_CACHE_PORT": dispatch_event_cache.cluster.attrRedisEndpointPort,
                    "DISPATCHED_ANALYZER_BUCKET": writes_to.bucketName,
                    "SUBGRAPH_MERGED_BUCKET": subgraphs_merged.bucket.bucketName,
                },
                vpc: vpc,
                reads_from: subgraphs_merged.bucket,
                subscribes_to: subgraphs_merged.topic,
                writes_to: writes_to,
            });

        service.readsFrom(analyzer_bucket, true);

        service.event_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
    }
}

class AnalyzerExecutor extends cdk.NestedStack {
    readonly count_cache: RedisCluster;
    readonly message_cache: RedisCluster;
    readonly hit_cache: RedisCluster;
    readonly bucket: s3.IBucket;
    readonly topic: sns.ITopic;

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        vpc: ec2.IVpc,
        reads_analyzers_from: s3.IBucket,
        writes_events_to: s3.IBucket,
        model_plugins_bucket: s3.IBucket,
        master_graph: DGraphEcs,
    ) {
        super(scope, id);

        const dispatched_analyzer = new EventEmitter(this, prefix + '-dispatched-analyzer');
        this.bucket = dispatched_analyzer.bucket;
        this.topic = dispatched_analyzer.topic;

        this.count_cache = new RedisCluster(this, 'ExecutorCountCache', vpc);
        this.hit_cache = new RedisCluster(this, 'ExecutorHitCache', vpc);
        this.message_cache = new RedisCluster(this, 'ExecutorMsgCache', vpc);

        const service = new Service(
            this,
            id,
            {
                environment: {
                    "ANALYZER_MATCH_BUCKET": writes_events_to.bucketName,
                    "BUCKET_PREFIX": prefix,
                    "MG_ALPHAS": master_graph.alphaNames.join(","),
                    "COUNTCACHE_ADDR": this.count_cache.cluster.attrRedisEndpointAddress,
                    "COUNTCACHE_PORT": this.count_cache.cluster.attrRedisEndpointPort,
                    "MESSAGECACHE_ADDR": this.message_cache.cluster.attrRedisEndpointAddress,
                    "MESSAGECACHE_PORT": this.message_cache.cluster.attrRedisEndpointPort,
                    "HITCACHE_ADDR": this.hit_cache.cluster.attrRedisEndpointAddress,
                    "HITCACHE_PORT": this.hit_cache.cluster.attrRedisEndpointPort,
                    "GRPC_ENABLE_FORK_SUPPORT": "1",
                },
                vpc: vpc,
                reads_from: dispatched_analyzer.bucket,
                writes_to: writes_events_to,
                subscribes_to: dispatched_analyzer.topic,
                opt: {
                    runtime: lambda.Runtime.PYTHON_3_7
                }
            });

        // We need the List capability to find each of the analyzers
        service.readsFrom(reads_analyzers_from, true);
        service.readsFrom(model_plugins_bucket, true);

        // Need to be able to GetObject in order to HEAD, can be replaced with
        // a cache later, but safe so long as there is no LIST
        let policy = new iam.PolicyStatement();
        policy.addActions('s3:GetObject');

        policy.addResources(writes_events_to.bucketArn);

        service.event_handler.addToRolePolicy(policy);
        service.event_retry_handler.addToRolePolicy(policy);

        service.event_handler.connections.allowToAnyIpv4(ec2.Port.allTraffic(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.allTraffic(), 'Allow outbound to S3');
    }
}

class EngagementCreator extends cdk.NestedStack {
    readonly bucket: s3.Bucket;

    constructor(
        scope: cdk.Construct,
        id: string,
        prefix: string,
        vpc: ec2.IVpc,
        publishes_to: sns.ITopic,
        master_graph: DGraphEcs,
    ) {
        super(scope, id);

        const analyzer_matched_sugraphs = new EventEmitter(this, prefix + '-analyzer-matched-subgraphs');
        this.bucket = analyzer_matched_sugraphs.bucket;

        const service = new Service(
            this,
            id,
            {
                environment: {
                    "MG_ALPHAS": master_graph.alphaNames.join(","),
                },
                vpc: vpc,
                reads_from: analyzer_matched_sugraphs.bucket,
                subscribes_to: analyzer_matched_sugraphs.topic,
                opt: {
                    runtime: lambda.Runtime.PYTHON_3_7
                }
            });

        service.publishesToTopic(publishes_to);

        service.event_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');

    }
}

class DGraphTtl extends cdk.Construct {

    constructor(
        scope: cdk.Construct,
        name: string,
        vpc: ec2.IVpc,
        master_graph: DGraphEcs
    ) {
        super(scope, name + "-stack");

        const grapl_version = process.env.GRAPL_VERSION || "latest";

        let event_handler = new lambda.Function(
            this, "Handler", {
                runtime: Runtime.PYTHON_3_7,
                handler: "app.prune_expired_subgraphs",
                functionName: "Grapl-DGraphTtl-Handler",
                code: lambda.Code.fromAsset(`./zips/dgraph-ttl-${grapl_version}.zip`),
                vpc: vpc,
                environment: {
                    "MG_ALPHAS": master_graph.alphaNames.join(","),
                    "GRAPL_DGRAPH_TTL_S": "2678400", // 60 * 60 * 24 * 31 == 1 month
                    "GRAPL_LOG_LEVEL": "INFO",
                    "GRAPL_TTL_DELETE_BATCH_SIZE": "1000"
                },
                timeout: cdk.Duration.seconds(600),
                memorySize: 128,
                description: grapl_version
            }
        );

        let target = new targets.LambdaFunction(event_handler);

        let rule = new events.Rule(
            scope, name + "-rule", {
                schedule: events.Schedule.expression("rate(1 hour)")
            }
        );
        rule.addTarget(target);
    }
}

class ModelPluginDeployer extends cdk.NestedStack {
    event_handler: lambda.Function;
    integration: apigateway.LambdaRestApi;
    name: string;
    integrationName: string;

    constructor(
        parent: cdk.Construct,
        name: string,
        prefix: string,
        jwt_secret: secretsmanager.Secret,
        master_graph: DGraphEcs,
        model_plugin_bucket: s3.IBucket,
        user_auth_table: UserAuthDb,
        vpc: ec2.Vpc,
    ) {
        super(parent, name + '-stack');

        this.name = name + prefix;
        this.integrationName = name + prefix + 'Integration';

        const grapl_version = process.env.GRAPL_VERSION || "latest";

        this.event_handler = new lambda.Function(
            this, 'Handler', {
                runtime: Runtime.PYTHON_3_7,
                handler: `grapl_model_plugin_deployer.app`,
                functionName: 'Grapl-ModelPluginDeployer-Handler',
                code: lambda.Code.fromAsset(`./zips/model-plugin-deployer-${grapl_version}.zip`),
                vpc: vpc,
                environment: {
                    "MG_ALPHAS": master_graph.alphaNames.join(","),
                    "JWT_SECRET_ID": jwt_secret.secretArn,
                    "USER_AUTH_TABLE": user_auth_table.user_auth_table.tableName,
                    "BUCKET_PREFIX": prefix
                },
                timeout: cdk.Duration.seconds(25),
                memorySize: 256,
                description: grapl_version,
            }
        );
        this.event_handler.currentVersion.addAlias('live');

        if (this.event_handler.role) {
            jwt_secret.grantRead(this.event_handler.role);
            user_auth_table.allowReadFromRole(this.event_handler.role);

            model_plugin_bucket.grantReadWrite(this.event_handler.role);
            model_plugin_bucket.grantDelete(this.event_handler.role);
        }

        this.integration = new apigateway.LambdaRestApi(
            this,
            this.integrationName,
            {
                handler: this.event_handler,
            },
        );

        this.integration.addUsagePlan('integrationApiUsagePlan', {
            quota: {
                limit: 1000,
                period: apigateway.Period.DAY,
            },
            throttle: {  // per minute
                rateLimit: 50,
                burstLimit: 50,
            }
        });
    }
}


export interface GraplEnvironementProps {
    prefix: string,
    jwt_secret: secretsmanager.Secret,
    vpc: ec2.IVpc,
    master_graph: DGraphEcs,
    user_auth_table: UserAuthDb,
}

export class GraplCdkStack extends cdk.Stack {
    grapl_env: GraplEnvironementProps;

    constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
        super(scope, id, props);

        const prefix = process.env.BUCKET_PREFIX || "my";

        const mgZeroCount = Number(process.env.MG_ZEROS_COUNT) || 1;
        const mgAlphaCount = Number(process.env.MG_ALPHAS_COUNT) || 1;

        const grapl_vpc = new ec2.Vpc(this, 'VPC', {
            natGateways: 1,
            enableDnsHostnames: true,
            enableDnsSupport: true,
        });


        const jwtSecret = new secretsmanager.Secret(this, 'EdgeJwtSecret', {
            description: 'The JWT secret that Grapl uses to authenticate its API',
            secretName: 'EdgeJwtSecret',
        });

        const user_auth_table = new UserAuthDb(this, 'UserAuthTable');

        const analyzers_bucket = new s3.Bucket(this, prefix + '-analyzers-bucket', {
            bucketName: prefix + '-analyzers-bucket',
            removalPolicy: cdk.RemovalPolicy.DESTROY,
            encryption: BucketEncryption.KMS_MANAGED,
            blockPublicAccess: BlockPublicAccess.BLOCK_ALL
        });

        const engagements_created_topic =
            new sns.Topic(this, id + '-engagements-created-topic', {
                topicName: 'engagements-created-topic'
            });

        const master_graph = new DGraphEcs(
            this,
            'master-graph',
            grapl_vpc,
            mgZeroCount,
            mgAlphaCount,
        );

        const engagement_creator = new EngagementCreator(
            this,
            'engagement-creator',
            prefix,
            grapl_vpc,
            engagements_created_topic,
            master_graph,
        );

        new DGraphTtl(
            this,
            "dgraph-ttl",
            grapl_vpc,
            master_graph
        );

        const model_plugins_bucket = new s3.Bucket(this, prefix + '-model-plugins-bucket', {
            bucketName: prefix + '-model-plugins-bucket',
            removalPolicy: cdk.RemovalPolicy.DESTROY,
        });

        new ModelPluginDeployer(
            this,
            'model-plugin-deployer',
            prefix,
            jwtSecret,
            master_graph,
            model_plugins_bucket,
            user_auth_table,
            grapl_vpc,
        );


        const analyzer_executor = new AnalyzerExecutor(
            this,
            'analyzer-executor',
            prefix,
            grapl_vpc,
            analyzers_bucket,
            engagement_creator.bucket,
            model_plugins_bucket,
            master_graph,
        );

        const analyzer_dispatch = new AnalyzerDispatch(
            this,
            'analyzer-dispatcher',
            prefix,
            grapl_vpc,
            analyzer_executor.bucket,
            analyzers_bucket,
        );

        const graph_merger = new GraphMerger(
            this,
            'graph-merger',
            prefix,
            grapl_vpc,
            analyzer_dispatch.bucket,
            master_graph,
        );

        const node_identifier = new NodeIdentifier(
            this,
            'node-identifier',
            prefix,
            grapl_vpc,
            graph_merger.bucket,
        );

        new SysmonSubgraphGenerator(
            this,
            'sysmon-subgraph-generator',
            prefix,
            grapl_vpc,
            node_identifier.bucket,
        );

        new EngagementNotebook(
            this,
            'engagements',
            prefix,
            user_auth_table,
            grapl_vpc,
        );

        this.grapl_env = {
            prefix,
            jwt_secret: jwtSecret,
            vpc: grapl_vpc,
            master_graph,
            user_auth_table,
        }
    }
}
