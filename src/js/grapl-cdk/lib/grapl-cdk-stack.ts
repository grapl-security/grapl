import * as cdk from '@aws-cdk/core';
import * as s3 from '@aws-cdk/aws-s3';
import * as sns from '@aws-cdk/aws-sns';
import * as sqs from '@aws-cdk/aws-sqs';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as events from '@aws-cdk/aws-events';
import * as targets from '@aws-cdk/aws-events-targets';
import * as lambda from '@aws-cdk/aws-lambda';
import * as iam from '@aws-cdk/aws-iam';
import * as apigateway from '@aws-cdk/aws-apigateway';
import * as secretsmanager from '@aws-cdk/aws-secretsmanager';
import * as route53 from '@aws-cdk/aws-route53';

import { Service } from './service';
import { UserAuthDb } from './userauthdb';
import { HistoryDb } from './historydb';
import { EventEmitter } from './event_emitters';
import { RedisCluster } from './redis';
import { EngagementNotebook } from './engagement';
import { EngagementEdge } from './engagement';
import { GraphQLEndpoint } from './graphql';
import { Swarm } from './swarm';

import { Watchful } from './vendor/cdk-watchful/lib/watchful';

interface SysmonGraphGeneratorProps extends GraplServiceProps {
    writesTo: s3.IBucket;
}

class SysmonGraphGenerator extends cdk.NestedStack {
    constructor(
        parent: cdk.Construct,
        id: string,
        props: SysmonGraphGeneratorProps
    ) {
        super(parent, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const sysmon_log = new EventEmitter(
            this,
            bucket_prefix + '-sysmon-log'
        );

        const event_cache = new RedisCluster(this, 'SysmonEventCache', props);
        event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                BUCKET_PREFIX: bucket_prefix,
                EVENT_CACHE_ADDR: event_cache.cluster.attrRedisEndpointAddress,
                EVENT_CACHE_PORT: event_cache.cluster.attrRedisEndpointPort,
            },
            vpc: props.vpc,
            reads_from: sysmon_log.bucket,
            subscribes_to: sysmon_log.topic,
            writes_to: props.writesTo,
            version: props.version,
            watchful: props.watchful,
        });

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(parseInt(event_cache.cluster.attrRedisEndpointPort))
        );

        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(parseInt(event_cache.cluster.attrRedisEndpointPort))
        );
    }
}

export interface NodeIdentifierProps extends GraplServiceProps {
    writesTo: s3.IBucket;
}

class NodeIdentifier extends cdk.NestedStack {
    readonly bucket: s3.Bucket;
    readonly topic: sns.Topic;

    constructor(parent: cdk.Construct, id: string, props: NodeIdentifierProps) {
        super(parent, id);

        const history_db = new HistoryDb(this, 'HistoryDB', props);

        const bucket_prefix = props.prefix.toLowerCase();
        const unid_subgraphs = new EventEmitter(
            this,
            bucket_prefix + '-unid-subgraphs-generated'
        );
        this.bucket = unid_subgraphs.bucket;
        this.topic = unid_subgraphs.topic;

        const retry_identity_cache = new RedisCluster(
            this,
            'NodeIdentifierRetryCache',
            props
        );
        retry_identity_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                BUCKET_PREFIX: bucket_prefix,
                RETRY_IDENTITY_CACHE_ADDR:
                    retry_identity_cache.cluster.attrRedisEndpointAddress,
                RETRY_IDENTITY_CACHE_PORT:
                    retry_identity_cache.cluster.attrRedisEndpointPort,
                STATIC_MAPPING_TABLE: history_db.static_mapping_table.tableName,
                DYNAMIC_SESSION_TABLE:
                    history_db.dynamic_session_table.tableName,
                PROCESS_HISTORY_TABLE: history_db.proc_history.tableName,
                FILE_HISTORY_TABLE: history_db.file_history.tableName,
                INBOUND_CONNECTION_HISTORY_TABLE:
                    history_db.inbound_connection_history.tableName,
                OUTBOUND_CONNECTION_HISTORY_TABLE:
                    history_db.outbound_connection_history.tableName,
                NETWORK_CONNECTION_HISTORY_TABLE:
                    history_db.network_connection_history.tableName,
                IP_CONNECTION_HISTORY_TABLE:
                    history_db.ip_connection_history.tableName,
                ASSET_ID_MAPPINGS: history_db.asset_history.tableName,
            },
            vpc: props.vpc,
            reads_from: unid_subgraphs.bucket,
            subscribes_to: unid_subgraphs.topic,
            writes_to: props.writesTo,
            retry_code_name: 'node-identifier-retry-handler',
            version: props.version,
            watchful: props.watchful,
        });

        history_db.allowReadWrite(service);

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(
                parseInt(retry_identity_cache.cluster.attrRedisEndpointPort)
            )
        );

        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(
                parseInt(retry_identity_cache.cluster.attrRedisEndpointPort)
            )
        );

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(443),
            'Allow outbound to S3'
        );
        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.tcp(443),
            'Allow outbound to S3'
        );
    }
}

export interface GraphMergerProps extends GraplServiceProps {
    writesTo: s3.IBucket;
}

class GraphMerger extends cdk.NestedStack {
    readonly bucket: s3.Bucket;

    constructor(scope: cdk.Construct, id: string, props: GraphMergerProps) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const subgraphs_generated = new EventEmitter(
            this,
            bucket_prefix + '-subgraphs-generated'
        );
        this.bucket = subgraphs_generated.bucket;

        const graph_merge_cache = new RedisCluster(
            this,
            'GraphMergerMergedCache',
            props
        );
        graph_merge_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                BUCKET_PREFIX: bucket_prefix,
                SUBGRAPH_MERGED_BUCKET: props.writesTo.bucketName,
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                MERGED_CACHE_ADDR:
                    graph_merge_cache.cluster.attrRedisEndpointAddress,
                MERGED_CACHE_PORT:
                    graph_merge_cache.cluster.attrRedisEndpointPort,
            },
            vpc: props.vpc,
            reads_from: subgraphs_generated.bucket,
            subscribes_to: subgraphs_generated.topic,
            writes_to: props.writesTo,
            version: props.version,
            watchful: props.watchful,
        });

        props.dgraphSwarmCluster.allowConnectionsFrom(service.event_handler);
    }
}

export interface AnalyzerDispatchProps extends GraplServiceProps {
    writesTo: s3.IBucket;
    readsFrom: s3.IBucket;
}

class AnalyzerDispatch extends cdk.NestedStack {
    readonly bucket: s3.Bucket;
    readonly topic: sns.Topic;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: AnalyzerDispatchProps
    ) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const subgraphs_merged = new EventEmitter(
            this,
            bucket_prefix + '-subgraphs-merged'
        );
        this.bucket = subgraphs_merged.bucket;
        this.topic = subgraphs_merged.topic;

        const dispatch_event_cache = new RedisCluster(
            this,
            'DispatchedEventCache',
            props
        );
        dispatch_event_cache.connections.allowFromAnyIpv4(ec2.Port.allTcp());

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                BUCKET_PREFIX: bucket_prefix,
                EVENT_CACHE_ADDR:
                    dispatch_event_cache.cluster.attrRedisEndpointAddress,
                EVENT_CACHE_PORT:
                    dispatch_event_cache.cluster.attrRedisEndpointPort,
                DISPATCHED_ANALYZER_BUCKET: props.writesTo.bucketName,
                SUBGRAPH_MERGED_BUCKET: subgraphs_merged.bucket.bucketName,
            },
            vpc: props.vpc,
            reads_from: subgraphs_merged.bucket,
            subscribes_to: subgraphs_merged.topic,
            writes_to: props.writesTo,
            version: props.version,
            watchful: props.watchful,
        });

        service.readsFrom(props.readsFrom, true);

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.allTcp(),
            'Allow outbound to S3'
        );
        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.allTcp(),
            'Allow outbound to S3'
        );
    }
}

export interface AnalyzerExecutorProps extends GraplServiceProps {
    writesTo: s3.IBucket;
    readsAnalyzersFrom: s3.IBucket;
    modelPluginsBucket: s3.IBucket;
}

class AnalyzerExecutor extends cdk.NestedStack {
    readonly bucket: s3.IBucket;

    constructor(
        scope: cdk.Construct,
        id: string,
        props: AnalyzerExecutorProps
    ) {
        super(scope, id);

        const bucket_prefix = props.prefix.toLowerCase();
        const dispatched_analyzer = new EventEmitter(
            this,
            bucket_prefix + '-dispatched-analyzer'
        );
        this.bucket = dispatched_analyzer.bucket;

        const count_cache = new RedisCluster(this, 'ExecutorCountCache', props);
        const hit_cache = new RedisCluster(this, 'ExecutorHitCache', props);
        const message_cache = new RedisCluster(this, 'ExecutorMsgCache', props);

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
                ANALYZER_MATCH_BUCKET: props.writesTo.bucketName,
                BUCKET_PREFIX: bucket_prefix,
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                COUNTCACHE_ADDR: count_cache.cluster.attrRedisEndpointAddress,
                COUNTCACHE_PORT: count_cache.cluster.attrRedisEndpointPort,
                MESSAGECACHE_ADDR:
                    message_cache.cluster.attrRedisEndpointAddress,
                MESSAGECACHE_PORT: message_cache.cluster.attrRedisEndpointPort,
                HITCACHE_ADDR: hit_cache.cluster.attrRedisEndpointAddress,
                HITCACHE_PORT: hit_cache.cluster.attrRedisEndpointPort,
                GRAPL_LOG_LEVEL: 'INFO',
                GRPC_ENABLE_FORK_SUPPORT: '1',
            },
            vpc: props.vpc,
            reads_from: dispatched_analyzer.bucket,
            writes_to: props.writesTo,
            subscribes_to: dispatched_analyzer.topic,
            opt: {
                runtime: lambda.Runtime.PYTHON_3_7,
            },
            version: props.version,
            watchful: props.watchful,
        });

        props.dgraphSwarmCluster.allowConnectionsFrom(service.event_handler);

        // We need the List capability to find each of the analyzers
        props.readsAnalyzersFrom.grantRead(service.event_handler);
        props.readsAnalyzersFrom.grantRead(service.event_retry_handler);

        service.readsFrom(props.modelPluginsBucket, true);

        // Need to be able to GetObject in order to HEAD, can be replaced with
        // a cache later, but safe so long as there is no LIST
        let policy = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: ['s3:GetObject'],
            resources: [props.writesTo.bucketArn + '/*'],
        });

        service.event_handler.addToRolePolicy(policy);
        service.event_retry_handler.addToRolePolicy(policy);

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.allTraffic(),
            'Allow outbound to S3'
        );
        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.allTraffic(),
            'Allow outbound to S3'
        );
    }
}

export interface EngagementCreatorProps extends GraplServiceProps {
    publishesTo: sns.ITopic;
}

class EngagementCreator extends cdk.NestedStack {
    readonly bucket: s3.Bucket;

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

        const service = new Service(this, id, {
            prefix: props.prefix,
            environment: {
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
        });

        props.dgraphSwarmCluster.allowConnectionsFrom(service.event_handler);

        service.publishesToTopic(props.publishesTo);

        service.event_handler.connections.allowToAnyIpv4(
            ec2.Port.allTcp(),
            'Allow outbound to S3'
        );
        service.event_retry_handler.connections.allowToAnyIpv4(
            ec2.Port.allTcp(),
            'Allow outbound to S3'
        );
    }
}

interface DGraphSwarmClusterProps {
    prefix: string;
    vpc: ec2.IVpc;
}

export class DGraphSwarmCluster extends cdk.NestedStack {
    private readonly dgraphAlphaZone: route53.PrivateHostedZone;
    private readonly dgraphSwarmCluster: Swarm;

    constructor(
        parent: cdk.Construct,
        id: string,
        props: DGraphSwarmClusterProps
    ) {
        super(parent, id);

        this.dgraphAlphaZone = new route53.PrivateHostedZone(
            this,
            'DGraphSwarmZone',
            {
                vpc: props.vpc,
                zoneName: 'alpha.dgraph.graplsecurity.com',
            }
        );

        this.dgraphSwarmCluster = new Swarm(this, 'DGraphSwarmCluster', {
            prefix: props.prefix,
            vpc: props.vpc,
            internalServicePorts: [ec2.Port.tcp(5080), ec2.Port.tcp(7080)],
        });
    }

    public alphaHostPort(): string {
        return `${this.dgraphAlphaZone.zoneName}:9080`;
    }

    public allowConnectionsFrom(other: ec2.IConnectable): void {
        this.dgraphSwarmCluster.allowConnectionsFrom(other, ec2.Port.tcp(9080));
    }
}

class DGraphTtl extends cdk.NestedStack {
    constructor(parent: cdk.Construct, id: string, props: GraplServiceProps) {
        super(parent, id);

        const serviceName = props.prefix + '-DGraphTtl';

        const role = new iam.Role(this, 'ExecutionRole', {
            assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
            roleName: serviceName + '-HandlerRole',
            description: 'Lambda execution role for: ' + serviceName,
            managedPolicies: [
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    'service-role/AWSLambdaBasicExecutionRole'
                ),
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    'service-role/AWSLambdaVPCAccessExecutionRole'
                ),
            ],
        });

        const event_handler = new lambda.Function(this, 'Handler', {
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: 'app.prune_expired_subgraphs',
            functionName: serviceName + '-Handler',
            code: lambda.Code.fromAsset(
                `./zips/dgraph-ttl-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                GRAPL_DGRAPH_TTL_S: '2678400', // 60 * 60 * 24 * 31 == 1 month
                GRAPL_LOG_LEVEL: 'INFO',
                GRAPL_TTL_DELETE_BATCH_SIZE: '1000',
            },
            timeout: cdk.Duration.seconds(600),
            memorySize: 128,
            description: props.version,
            role,
        });
        event_handler.currentVersion.addAlias('live');

        props.dgraphSwarmCluster.allowConnectionsFrom(event_handler);

        const target = new targets.LambdaFunction(event_handler);

        const rule = new events.Rule(this, 'Rule', {
            schedule: events.Schedule.expression('rate(1 hour)'),
        });
        rule.addTarget(target);

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                event_handler.functionName,
                event_handler
            );
        }
    }
}

export interface ModelPluginDeployerProps extends GraplServiceProps {
    modelPluginBucket: s3.IBucket;
}

export class ModelPluginDeployer extends cdk.NestedStack {
    integrationName: string;

    constructor(
        parent: cdk.Construct,
        id: string,
        props: ModelPluginDeployerProps
    ) {
        super(parent, id);

        const serviceName = props.prefix + '-ModelPluginDeployer';
        this.integrationName = id + props.prefix + 'Integration';
        const ux_bucket = s3.Bucket.fromBucketName(
            this,
            'uxBucket',
            props.prefix.toLowerCase() + '-engagement-ux-bucket'
        );

        const role = new iam.Role(this, 'ExecutionRole', {
            assumedBy: new iam.ServicePrincipal('lambda.amazonaws.com'),
            roleName: serviceName + '-HandlerRole',
            description: 'Lambda execution role for: ' + serviceName,
            managedPolicies: [
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    'service-role/AWSLambdaBasicExecutionRole'
                ),
                iam.ManagedPolicy.fromAwsManagedPolicyName(
                    'service-role/AWSLambdaVPCAccessExecutionRole'
                ),
            ],
        });

        const event_handler = new lambda.Function(this, 'Handler', {
            runtime: lambda.Runtime.PYTHON_3_7,
            handler: `grapl_model_plugin_deployer.app`,
            functionName: serviceName + '-Handler',
            code: lambda.Code.fromAsset(
                `./zips/model-plugin-deployer-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
                JWT_SECRET_ID: props.jwtSecret.secretArn,
                USER_AUTH_TABLE: props.userAuthTable.user_auth_table.tableName,
                BUCKET_PREFIX: props.prefix,
                UX_BUCKET_URL: 'https://' + ux_bucket.bucketRegionalDomainName,
                GRAPL_LOG_LEVEL: 'DEBUG',
            },
            timeout: cdk.Duration.seconds(25),
            memorySize: 256,
            description: props.version,
            role,
        });
        event_handler.currentVersion.addAlias('live');

        props.dgraphSwarmCluster.allowConnectionsFrom(event_handler);

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                event_handler.functionName,
                event_handler
            );
        }

        if (event_handler.role) {
            props.jwtSecret.grantRead(event_handler.role);
            props.userAuthTable.allowReadFromRole(event_handler.role);

            props.modelPluginBucket.grantReadWrite(event_handler.role);
            props.modelPluginBucket.grantDelete(event_handler.role);
        }

        const integration = new apigateway.LambdaRestApi(this, 'Integration', {
            restApiName: this.integrationName,
            endpointExportName: serviceName + '-EndpointApi',
            handler: event_handler,
        });

        integration.addUsagePlan('integrationApiUsagePlan', {
            quota: {
                limit: 1000,
                period: apigateway.Period.DAY,
            },
            throttle: {
                // per minute
                rateLimit: 50,
                burstLimit: 50,
            },
        });

        if (props.watchful) {
            props.watchful.watchApiGateway(
                serviceName + '-Integration',
                integration,
                {
                    serverErrorThreshold: 1, // any 5xx alerts
                    cacheGraph: true,
                    watchedOperations: [
                        {
                            httpMethod: 'POST',
                            resourcePath: '/gitWebhook',
                        },
                        {
                            httpMethod: 'OPTIONS',
                            resourcePath: '/gitWebHook',
                        },
                        {
                            httpMethod: 'POST',
                            resourcePath: '/deploy',
                        },
                        {
                            httpMethod: 'OPTIONS',
                            resourcePath: '/deploy',
                        },
                        {
                            httpMethod: 'POST',
                            resourcePath: '/listModelPlugins',
                        },
                        {
                            httpMethod: 'OPTIONS',
                            resourcePath: '/listModelPlugins',
                        },
                        {
                            httpMethod: 'POST',
                            resourcePath: '/deleteModelPlugin',
                        },
                        {
                            httpMethod: 'OPTIONS',
                            resourcePath: '/deleteModelPlugin',
                        },
                        {
                            httpMethod: 'POST',
                            resourcePath: '/{proxy+}',
                        },
                        {
                            httpMethod: 'OPTIONS',
                            resourcePath: '/{proxy+}',
                        },
                    ],
                }
            );
        }
    }
}

export interface GraplServiceProps {
    prefix: string;
    version: string;
    jwtSecret: secretsmanager.Secret;
    vpc: ec2.IVpc;
    dgraphSwarmCluster: DGraphSwarmCluster;
    userAuthTable: UserAuthDb;
    watchful?: Watchful;
}

export interface GraplStackProps extends cdk.StackProps {
    stackName: string;
    version?: string;
    watchfulEmail?: string;
}

export class GraplCdkStack extends cdk.Stack {
    prefix: string;
    engagement_edge: EngagementEdge;
    graphql_endpoint: GraphQLEndpoint;
    model_plugin_deployer: ModelPluginDeployer;

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
            description:
                'The JWT secret that Grapl uses to authenticate its API',
            secretName: this.prefix + '-EdgeJwtSecret',
        });

        const user_auth_table = new UserAuthDb(this, 'UserAuthTable', {
            table_name: this.prefix.toLowerCase() + '-user_auth_table',
        });

        let watchful = undefined;
        if (props.watchfulEmail) {
            const alarmSqs = new sqs.Queue(this, 'alarmSqs');
            const alarmSns = new sns.Topic(this, 'alarmSns');

            watchful = new Watchful(this, id + '-Watchful', {
                alarmEmail: props.watchfulEmail,
                alarmSqs,
                alarmSns,
            });
        }

        const dgraphSwarmCluster = new DGraphSwarmCluster(
            this,
            'dgraph-swarm-cluster',
            {
                prefix: this.prefix,
                vpc: grapl_vpc,
            }
        );

        const graplProps = {
            prefix: this.prefix,
            version: props.version || 'latest',
            jwtSecret: jwtSecret,
            vpc: grapl_vpc,
            dgraphSwarmCluster: dgraphSwarmCluster,
            userAuthTable: user_auth_table,
            watchful: watchful,
        };

        const analyzers_bucket = new s3.Bucket(this, 'AnalyzersBucket', {
            bucketName: bucket_prefix + '-analyzers-bucket',
            removalPolicy: cdk.RemovalPolicy.DESTROY,
            encryption: s3.BucketEncryption.KMS_MANAGED,
            blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
        });

        const engagements_created_topic = new sns.Topic(
            this,
            'EngagementsCreatedTopic',
            {
                topicName: this.prefix + '-engagements-created-topic',
            }
        );

        const engagement_creator = new EngagementCreator(
            this,
            'engagement-creator',
            {
                publishesTo: engagements_created_topic,
                ...graplProps,
            }
        );

        new DGraphTtl(this, 'dgraph-ttl', graplProps);

        const model_plugins_bucket = new s3.Bucket(this, 'ModelPluginsBucket', {
            bucketName: bucket_prefix + '-model-plugins-bucket',
            removalPolicy: cdk.RemovalPolicy.DESTROY,
        });

        this.model_plugin_deployer = new ModelPluginDeployer(
            this,
            'model-plugin-deployer',
            {
                modelPluginBucket: model_plugins_bucket,
                ...graplProps,
            }
        );

        const analyzer_executor = new AnalyzerExecutor(
            this,
            'analyzer-executor',
            {
                writesTo: engagement_creator.bucket,
                readsAnalyzersFrom: analyzers_bucket,
                modelPluginsBucket: model_plugins_bucket,
                ...graplProps,
            }
        );

        const analyzer_dispatch = new AnalyzerDispatch(
            this,
            'analyzer-dispatcher',
            {
                writesTo: analyzer_executor.bucket,
                readsFrom: analyzers_bucket,
                ...graplProps,
            }
        );

        const graph_merger = new GraphMerger(this, 'graph-merger', {
            writesTo: analyzer_dispatch.bucket,
            ...graplProps,
        });

        const node_identifier = new NodeIdentifier(this, 'node-identifier', {
            writesTo: graph_merger.bucket,
            ...graplProps,
        });

        new SysmonGraphGenerator(this, 'sysmon-subgraph-generator', {
            writesTo: node_identifier.bucket,
            ...graplProps,
        });

        new EngagementNotebook(this, 'engagements', {
            model_plugins_bucket,
            ...graplProps,
        });

        this.engagement_edge = new EngagementEdge(
            this,
            'EngagementEdge',
            graplProps
        );

        const ux_bucket = new s3.Bucket(this, 'EdgeBucket', {
            bucketName:
                graplProps.prefix.toLowerCase() + '-engagement-ux-bucket',
            publicReadAccess: true,
            websiteIndexDocument: 'index.html',
            removalPolicy: cdk.RemovalPolicy.DESTROY,
        });

        this.graphql_endpoint = new GraphQLEndpoint(
            this,
            'GraphqlEndpoint',
            graplProps,
            ux_bucket
        );
    }
}
