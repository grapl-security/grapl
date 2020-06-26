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
import { EngagementEdge } from './engagement';
import { GraphQLEndpoint } from './graphql';

interface SysmonGraphGeneratorProps extends GraplServiceProps {
    writesTo: s3.IBucket,
}

class SysmonGraphGenerator extends cdk.Construct {

    constructor(
        scope: cdk.Construct,
        id: string,
        props: SysmonGraphGeneratorProps,
    ) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const sysmon_log = new EventEmitter(this, bucket_prefix + '-sysmon-log');

        const event_cache = new RedisCluster(this, 'SysmonEventCache', props);
        event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                "BUCKET_PREFIX": bucket_prefix,
                "EVENT_CACHE_ADDR": event_cache.cluster.attrRedisEndpointAddress,
                "EVENT_CACHE_PORT": event_cache.cluster.attrRedisEndpointPort,
            },
            vpc: props.vpc,
            reads_from: sysmon_log.bucket,
            subscribes_to: sysmon_log.topic,
            writes_to: props.writesTo,
            version: props.version,
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

export interface NodeIdentifierProps extends GraplServiceProps {
    writesTo: s3.IBucket,
}

class NodeIdentifier extends cdk.Construct {
    readonly bucket: s3.Bucket;
    readonly topic: sns.Topic;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: NodeIdentifierProps,
    ) {
        super(scope, id);

        const history_db = new HistoryDb(this, 'HistoryDB', props);

        const bucket_prefix = props.prefix.toLowerCase();
        const unid_subgraphs = new EventEmitter(this, bucket_prefix + '-unid-subgraphs-generated');
        this.bucket = unid_subgraphs.bucket;
        this.topic = unid_subgraphs.topic;

        const retry_identity_cache = new RedisCluster(this, 'NodeIdentifierRetryCache', props);
        retry_identity_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                "BUCKET_PREFIX": bucket_prefix,
                "RETRY_IDENTITY_CACHE_ADDR": retry_identity_cache.cluster.attrRedisEndpointAddress,
                "RETRY_IDENTITY_CACHE_PORT": retry_identity_cache.cluster.attrRedisEndpointPort,
                "STATIC_MAPPING_TABLE": history_db.static_mapping_table.tableName,
                "DYNAMIC_SESSION_TABLE": history_db.DynamicSessionTable.tableName,
                "PROCESS_HISTORY_TABLE": history_db.ProcessHistoryTable.tableName,
                "FILE_HISTORY_TABLE": history_db.FileHistoryTable.tableName,
                "INBOUND_CONNECTION_HISTORY_TABLE": history_db.InboundConnectionHistoryTable.tableName,
                "OUTBOUND_CONNECTION_HISTORY_TABLE": history_db.OutboundConnectionHistoryTable.tableName,
                "NETWORK_CONNECTION_HISTORY_TABLE": history_db.NetworkConnectionHistoryTable.tableName,
                "IP_CONNECTION_HISTORY_TABLE": history_db.IpConnectionHistoryTable.tableName,
            },
            vpc: props.vpc,
            reads_from: unid_subgraphs.bucket,
            subscribes_to: unid_subgraphs.topic,
            writes_to: props.writesTo,
            retry_code_name: 'node-identifier-retry-handler',
            version: props.version,
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

export interface GraphMergerProps extends GraplServiceProps {
    writesTo: s3.IBucket,
}

class GraphMerger extends cdk.NestedStack {
    readonly bucket: s3.Bucket;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: GraphMergerProps,
    ) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const subgraphs_generated = new EventEmitter(this, bucket_prefix + '-subgraphs-generated');
        this.bucket = subgraphs_generated.bucket;

        const graph_merge_cache = new RedisCluster(this, 'GraphMergerMergedCache', props);
        graph_merge_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        new Service(this, id, {
            prefix: props.prefix,
            environment: {
                "BUCKET_PREFIX": bucket_prefix,
                "SUBGRAPH_MERGED_BUCKET": props.writesTo.bucketName,
                "MG_ALPHAS": props.masterGraph.alphaHostPorts().join(","),
                "MERGED_CACHE_ADDR": graph_merge_cache.cluster.attrRedisEndpointAddress,
                "MERGED_CACHE_PORT": graph_merge_cache.cluster.attrRedisEndpointPort,
            },
            vpc: props.vpc,
            reads_from: subgraphs_generated.bucket,
            subscribes_to: subgraphs_generated.topic,
            writes_to: props.writesTo,
            version: props.version,
        });
    }
}

export interface AnalyzerDispatchProps extends GraplServiceProps {
    writesTo: s3.IBucket,
    readsFrom: s3.IBucket,
}

class AnalyzerDispatch extends cdk.NestedStack {
    readonly bucket: s3.Bucket;
    readonly topic: sns.Topic;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: AnalyzerDispatchProps,
    ) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const subgraphs_merged = new EventEmitter(this, bucket_prefix + '-subgraphs-merged');
        this.bucket = subgraphs_merged.bucket;
        this.topic = subgraphs_merged.topic;

        const dispatch_event_cache = new RedisCluster(this, 'DispatchedEventCache', props);
        dispatch_event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                "BUCKET_PREFIX": bucket_prefix,
                "EVENT_CACHE_ADDR": dispatch_event_cache.cluster.attrRedisEndpointAddress,
                "EVENT_CACHE_PORT": dispatch_event_cache.cluster.attrRedisEndpointPort,
                "DISPATCHED_ANALYZER_BUCKET": props.writesTo.bucketName,
                "SUBGRAPH_MERGED_BUCKET": subgraphs_merged.bucket.bucketName,
            },
            vpc: props.vpc,
            reads_from: subgraphs_merged.bucket,
            subscribes_to: subgraphs_merged.topic,
            writes_to: props.writesTo,
            version: props.version,
        });

        service.readsFrom(props.readsFrom, true);

        service.event_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
    }
}

export interface AnalyzerExecutorProps extends GraplServiceProps {
    writesTo: s3.IBucket,
    readsAnalyzersFrom: s3.IBucket,
    modelPluginsBucket: s3.IBucket,
}

class AnalyzerExecutor extends cdk.NestedStack {
    readonly bucket: s3.IBucket;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: AnalyzerExecutorProps,
    ) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const dispatched_analyzer = new EventEmitter(this, bucket_prefix + '-dispatched-analyzer');
        this.bucket = dispatched_analyzer.bucket;

        const count_cache = new RedisCluster(this, 'ExecutorCountCache', props);
        const hit_cache = new RedisCluster(this, 'ExecutorHitCache', props);
        const message_cache = new RedisCluster(this, 'ExecutorMsgCache', props);

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                "ANALYZER_MATCH_BUCKET": props.writesTo.bucketName,
                "BUCKET_PREFIX": bucket_prefix,
                "MG_ALPHAS": props.masterGraph.alphaHostPorts().join(","),
                "COUNTCACHE_ADDR": count_cache.cluster.attrRedisEndpointAddress,
                "COUNTCACHE_PORT": count_cache.cluster.attrRedisEndpointPort,
                "MESSAGECACHE_ADDR": message_cache.cluster.attrRedisEndpointAddress,
                "MESSAGECACHE_PORT": message_cache.cluster.attrRedisEndpointPort,
                "HITCACHE_ADDR": hit_cache.cluster.attrRedisEndpointAddress,
                "HITCACHE_PORT": hit_cache.cluster.attrRedisEndpointPort,
                "GRPC_ENABLE_FORK_SUPPORT": "1",
            },
            vpc: props.vpc,
            reads_from: dispatched_analyzer.bucket,
            writes_to: props.writesTo,
            subscribes_to: dispatched_analyzer.topic,
            opt: {
                runtime: lambda.Runtime.PYTHON_3_7
            },
            version: props.version,
        });

        // We need the List capability to find each of the analyzers
        service.readsFrom(props.readsAnalyzersFrom, true);
        service.readsFrom(props.modelPluginsBucket, true);

        // Need to be able to GetObject in order to HEAD, can be replaced with
        // a cache later, but safe so long as there is no LIST
        let policy = new iam.PolicyStatement();
        policy.addActions('s3:GetObject');

        policy.addResources(props.writesTo.bucketArn);

        service.event_handler.addToRolePolicy(policy);
        service.event_retry_handler.addToRolePolicy(policy);

        service.event_handler.connections.allowToAnyIpv4(ec2.Port.allTraffic(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.allTraffic(), 'Allow outbound to S3');
    }
}

export interface EngagementCreatorProps extends GraplServiceProps {
    publishesTo: sns.ITopic,
}

class EngagementCreator extends cdk.NestedStack {
    readonly bucket: s3.Bucket;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: EngagementCreatorProps,
    ) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const analyzer_matched_sugraphs = new EventEmitter(this, bucket_prefix + '-analyzer-matched-subgraphs');
        this.bucket = analyzer_matched_sugraphs.bucket;

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                "MG_ALPHAS": props.masterGraph.alphaHostPorts().join(","),
            },
            vpc: props.vpc,
            reads_from: analyzer_matched_sugraphs.bucket,
            subscribes_to: analyzer_matched_sugraphs.topic,
            opt: {
                runtime: lambda.Runtime.PYTHON_3_7
            },
            version: props.version,
        });

        service.publishesToTopic(props.publishesTo);

        service.event_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');
        service.event_retry_handler.connections.allowToAnyIpv4(ec2.Port.allTcp(), 'Allow outbound to S3');

    }
}

class DGraphTtl extends cdk.Construct {

    constructor(
        scope: cdk.Construct,
        id: string,
        props: GraplServiceProps,
    ) {
        super(scope, id);

        const serviceName = props.prefix + '-DGraphTtl';

        const event_handler = new lambda.Function(
            this, "Handler", {
                runtime: Runtime.PYTHON_3_7,
                handler: "app.prune_expired_subgraphs",
                functionName: serviceName + "-Handler",
                code: lambda.Code.fromAsset(`./zips/dgraph-ttl-${props.version}.zip`),
                vpc: props.vpc,
                environment: {
                    "MG_ALPHAS": props.masterGraph.alphaHostPorts().join(","),
                    "GRAPL_DGRAPH_TTL_S": "2678400", // 60 * 60 * 24 * 31 == 1 month
                    "GRAPL_LOG_LEVEL": "INFO",
                    "GRAPL_TTL_DELETE_BATCH_SIZE": "1000"
                },
                timeout: cdk.Duration.seconds(600),
                memorySize: 128,
                description: props.version,
            }
        );
        event_handler.currentVersion.addAlias('live');

        const target = new targets.LambdaFunction(event_handler);

        const rule = new events.Rule(
            scope, 'Rule', {
                schedule: events.Schedule.expression("rate(1 hour)")
            }
        );
        rule.addTarget(target);
    }
}

export interface ModelPluginDeployerProps extends GraplServiceProps {
    modelPluginBucket: s3.IBucket,
}

class ModelPluginDeployer extends cdk.NestedStack {

    constructor(
        parent: cdk.Construct,
        id: string,
        props: ModelPluginDeployerProps,
    ) {
        super(parent, id);

        const serviceName = props.prefix + '-ModelPluginDeployer';

        const event_handler = new lambda.Function(
            this, 'Handler', {
                runtime: Runtime.PYTHON_3_7,
                handler: `grapl_model_plugin_deployer.app`,
                functionName: serviceName + '-Handler',
                code: lambda.Code.fromAsset(`./zips/model-plugin-deployer-${props.version}.zip`),
                vpc: props.vpc,
                environment: {
                    "MG_ALPHAS": props.masterGraph.alphaHostPorts().join(","),
                    "JWT_SECRET_ID": props.jwtSecret.secretArn,
                    "USER_AUTH_TABLE": props.userAuthTable.user_auth_table.tableName,
                    "BUCKET_PREFIX": props.prefix,
                },
                timeout: cdk.Duration.seconds(25),
                memorySize: 256,
                description: props.version,
            }
        );
        event_handler.currentVersion.addAlias('live');

        if (event_handler.role) {
            props.jwtSecret.grantRead(event_handler.role);
            props.userAuthTable.allowReadFromRole(event_handler.role);

            props.modelPluginBucket.grantReadWrite(event_handler.role);
            props.modelPluginBucket.grantDelete(event_handler.role);
        }

        const integration = new apigateway.LambdaRestApi(
            this,
            'Integration',
            {
                restApiName: serviceName + '-Integration',
                handler: event_handler,
            },
        );

        integration.addUsagePlan('integrationApiUsagePlan', {
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

export interface GraplServiceProps {
    prefix: string,
    version: string,
    jwtSecret: secretsmanager.Secret,
    vpc: ec2.IVpc,
    masterGraph: DGraphEcs,
    userAuthTable: UserAuthDb,
}

export interface GraplStackProps extends cdk.StackProps {
    stackName: string,
    version?: string,
    graphAlphaCount?: number,
    graphAlphaPort?: number,
    graphZeroCount?: number,
}

export class GraplCdkStack extends cdk.Stack {
    prefix: string;
    engagement_edge: EngagementEdge;
    graphql_endpoint: GraphQLEndpoint;

    constructor(scope: cdk.Construct, id: string, props: GraplStackProps) {
        super(scope, id, props);

        this.prefix = props.stackName;
        const bucket_prefix = this.prefix.toLowerCase();

        const grapl_vpc = new ec2.Vpc(this, this.prefix + '-VPC', {
            natGateways: 1,
            enableDnsHostnames: true,
            enableDnsSupport: true,
        });

        const jwtSecret = new secretsmanager.Secret(this, 'EdgeJwtSecret', {
            description: 'The JWT secret that Grapl uses to authenticate its API',
            secretName: this.prefix + '-EdgeJwtSecret',
        });

        const user_auth_table = new UserAuthDb(this, 'UserAuthTable', {
            table_name: this.prefix.toLowerCase() + '-user_auth_table'
        });

        const master_graph = new DGraphEcs(
            this,
            'master-graph', {
                prefix: this.prefix,
                vpc: grapl_vpc,
                alphaCount: props.graphZeroCount || 1,
                alphaPort: props.graphAlphaPort || 9080,
                zeroCount: props.graphAlphaCount || 1,
            }
        );

        const graplProps = {
            prefix: this.prefix,
            version: props.version || 'latest',
            jwtSecret: jwtSecret,
            vpc: grapl_vpc,
            masterGraph: master_graph,
            userAuthTable: user_auth_table,
        }

        const analyzers_bucket = new s3.Bucket(this, 'AnalyzersBucket', {
            bucketName: bucket_prefix + '-analyzers-bucket',
            removalPolicy: cdk.RemovalPolicy.DESTROY,
            encryption: BucketEncryption.KMS_MANAGED,
            blockPublicAccess: BlockPublicAccess.BLOCK_ALL
        });

        const engagements_created_topic =
            new sns.Topic(this, 'EngagementsCreatedTopic', {
                topicName: this.prefix + '-engagements-created-topic'
            });

        const engagement_creator = new EngagementCreator(
            this,
            'engagement-creator', {
                publishesTo: engagements_created_topic,
                ...graplProps,
            },
        );

        new DGraphTtl(
            this,
            'dgraph-ttl',
            graplProps,
        );

        const model_plugins_bucket = new s3.Bucket(this, 'ModelPluginsBucket', {
            bucketName: bucket_prefix + '-model-plugins-bucket',
            removalPolicy: cdk.RemovalPolicy.DESTROY,
        });

        new ModelPluginDeployer(
            this,
            'model-plugin-deployer', {
                modelPluginBucket: model_plugins_bucket,
                ...graplProps
            },
        );

        const analyzer_executor = new AnalyzerExecutor(
            this,
            'analyzer-executor', {
                writesTo: engagement_creator.bucket,
                readsAnalyzersFrom: analyzers_bucket,
                modelPluginsBucket: model_plugins_bucket,
                ...graplProps
            },
        );

        const analyzer_dispatch = new AnalyzerDispatch(
            this,
            'analyzer-dispatcher', {
                writesTo: analyzer_executor.bucket,
                readsFrom: analyzers_bucket,
                ...graplProps
            },
        );

        const graph_merger = new GraphMerger(
            this,
            'graph-merger', {
                writesTo: analyzer_dispatch.bucket,
                ...graplProps,
            },
        );

        const node_identifier = new NodeIdentifier(
            this,
            'node-identifier', {
                writesTo: graph_merger.bucket,
                ...graplProps,
            }
        );

        new SysmonGraphGenerator(
            this,
            'sysmon-subgraph-generator', {
                writesTo: node_identifier.bucket,
                ...graplProps
            },
        );

        new EngagementNotebook(
            this,
            'engagements',
            graplProps,
        );

        this.engagement_edge = new EngagementEdge(
            this,
            'EngagementEdge',
            graplProps
         );

        this.graphql_endpoint = new GraphQLEndpoint(
            this,
            'GraphqlEndpoint',
            graplProps,
        );
    }
}
