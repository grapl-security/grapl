import * as cdk from "@aws-cdk/core";
import * as lambda from "@aws-cdk/aws-lambda";
import * as s3 from "@aws-cdk/aws-s3";
import * as ec2 from "@aws-cdk/aws-ec2";
import {RedisCluster} from "../../../grapl-cdk/lib/redis";
import {DGraphEcs} from "../../../grapl-cdk/lib/dgraph";
import * as sns from "@aws-cdk/aws-sns";
import {IVpc, Vpc} from "@aws-cdk/aws-ec2";

type NetworkPermissions = {
  // Either allows all external networking, or to a set of (IP ranges + Ports)
  // default: false
  external_networking: boolean | [string, number][],
};

type AnalyzerPermissions = {
  network: NetworkPermissions,
  // default, []
  requires_kv_store: ("read" | "write" | "delete")[]
};

type AnalyzerMeta = {
  analyzer_name: string,
  analyzer_version: string,
  entrypoint: string,
  timeout: number, // default 180
  memorySize: number, // default 256
  analyzer_permissions: AnalyzerPermissions,
};

type RedisClusterConfig = {
  endpointAddress: string,
  endpointPort: number,
};

type AnalyzerCaches = {
  count_cache: RedisClusterConfig,
  message_cache: RedisClusterConfig,
  hit_cache: RedisClusterConfig,
}

class Analyzer extends cdk.NestedStack {
  constructor(
      scope: cdk.Construct,
      id: string,
      prefix: string,
      grapl_version: string,
      analyzer_meta: AnalyzerMeta,
      analyzer_caches: AnalyzerCaches,
      vpc: ec2.IVpc,
      merged_graph_bucket: s3.IBucket,
      analyzer_match_bucket: s3.IBucket,
      model_plugins_bucket: s3.IBucket,
      master_graph_alphas: string[],
  ) {
    super(scope, id);

    const event_handler = new lambda.Function(
        scope, 'Handler',
        {
          runtime: lambda.Runtime.PYTHON_3_7,
          handler: analyzer_meta.entrypoint + ".grapl_analyzer",
          functionName: `Grapl-${analyzer_meta.analyzer_name}-Handler`,
          code: lambda.Code.fromAsset(`./zips/${analyzer_meta.analyzer_name}-${analyzer_meta.analyzer_version}.zip`),
          vpc,
          environment: {
            IS_RETRY: "False",
            ANALYZER_MATCH_BUCKET: analyzer_match_bucket.bucketName,
            BUCKET_PREFIX: prefix,
            MG_ALPHAS: master_graph_alphas.join(","),
            COUNTCACHE_ADDR: analyzer_caches.count_cache.endpointAddress,
            COUNTCACHE_PORT: analyzer_caches.count_cache.endpointPort.toString(),
            MESSAGECACHE_ADDR: analyzer_caches.message_cache.endpointAddress,
            MESSAGECACHE_PORT: analyzer_caches.message_cache.endpointPort.toString(),
            HITCACHE_ADDR: analyzer_caches.hit_cache.endpointAddress,
            HITCACHE_PORT: analyzer_caches.hit_cache.endpointPort.toString(),
            GRPC_ENABLE_FORK_SUPPORT: "1",
          },
          timeout: cdk.Duration.seconds(analyzer_meta.timeout),
          memorySize: analyzer_meta.memorySize,
          description: grapl_version,
        });

    event_handler.currentVersion.addAlias('live');
  }
}

const parse_analyzer_meta = (analyzer_toml: string): AnalyzerMeta => {
  return {} as any as AnalyzerMeta
}

const loadTomls = (directory: string): string[] => {
  return []
}

const vpc_from_prefix = (scope: cdk.Stack, prefix: string, vpcId: string): IVpc => {
  return Vpc.fromLookup(
      scope,
      prefix + 'Vpc',
      {
        vpcId,
      }
  );
}

const load_analyzer_caches = (): AnalyzerCaches => {
  return {} as any as AnalyzerCaches
}

const assert = <T>(message: string, arg: T | undefined): T => {
  if (!arg) {
    throw new Error(message)
  }
  return arg;
}

export class AnalyzerCdkStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);
    const prefix = assert(
        "CUSTOMER_PREFIX",
        process.env.CUSTOMER_PREFIX,
    );

    const grapl_version = assert(
        "GRAPL_VERSION",
        process.env.GRAPL_VERSION,
    );

    const vpc_id = assert(
        "VPC_ID",
        process.env.VPC_ID,
    );

    const merged_graph_bucket_arn = assert(
        "MERGED_GRAPH_BUCKET_ARN",
        process.env.MERGED_GRAPH_BUCKET_ARN,
    );

    const analyzer_matched_bucket_arn = assert(
        "ANALYZER_MATCHED_BUCKET_ARN",
        process.env.ANALYZER_MATCHED_BUCKET_ARN,
    );

    const model_plugin_bucket_arn = assert(
        "MODEL_PLUGIN_BUCKET_ARN",
        process.env.MODEL_PLUGIN_BUCKET_ARN,
    );

    const master_graph_alphas = assert(
        "MG_ALPHAS",
        process.env.MG_ALPHAS,
    ).split(",");

    const vpc = vpc_from_prefix(this, prefix, vpc_id);

    const merged_graph_bucket = s3.Bucket.fromBucketArn(
        scope,
        'mergedGraphBucket',
        merged_graph_bucket_arn,
    );

    const analyzer_matched_bucket = s3.Bucket.fromBucketArn(
        scope,
        'analyzerMatchedBucket',
        analyzer_matched_bucket_arn,
    );

    // TODO: We should use a lambda layer for model plugins, but for now this will do
    const model_plugin_bucket = s3.Bucket.fromBucketArn(
        scope,
        'modelPluginBucketBucket',
        model_plugin_bucket_arn,
    );

    const analyzer_caches = load_analyzer_caches();

    for (const analyzer_toml of loadTomls("./zips/")) {
      const analyzer_meta = parse_analyzer_meta(analyzer_toml);

      new Analyzer(
          this,
          analyzer_meta.analyzer_name + 'executor',
          prefix,
          grapl_version,
          analyzer_meta,
          analyzer_caches,
          vpc,
          merged_graph_bucket,
          analyzer_matched_bucket,
          model_plugin_bucket,
          master_graph_alphas,
      );
    }


  }
}
