import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as elasticache from '@aws-cdk/aws-elasticache';

import { GraplServiceProps } from './grapl-cdk-stack';

export class RedisCluster extends cdk.Construct {
    readonly securityGroup: ec2.SecurityGroup;
    readonly connections: ec2.Connections;
    readonly cluster: elasticache.CfnCacheCluster;
    readonly address: string;

    constructor(scope: cdk.Construct, id: string, props: GraplServiceProps) {
        super(scope, id);

        // Define a group for telling Elasticache which subnets to put cache nodes in.
        const subnetGroup = new elasticache.CfnSubnetGroup(
            this,
            'ElasticacheSubnetGroup',
            {
                description: `List of subnets used for redis cache ${props.deploymentName}-${id}`,
                subnetIds: props.vpc.privateSubnets.map(
                    (subnet) => subnet.subnetId
                ),
                cacheSubnetGroupName: `${props.deploymentName}-${id}-cache-subnet-group`,
            }
        );

        // The security group that defines network level access to the cluster
        this.securityGroup = new ec2.SecurityGroup(this, 'Ec2SubnetGroup', {
            vpc: props.vpc,
        });

        this.connections = new ec2.Connections({
            securityGroups: [this.securityGroup],
            defaultPort: ec2.Port.tcp(6379),
        });

        this.connections.allowFromAnyIpv4(ec2.Port.tcp(6379));

        // The cluster resource itself.
        this.cluster = new elasticache.CfnCacheCluster(this, 'Cluster', {
            clusterName: `${props.deploymentName}-redis-${id}`,
            cacheNodeType: 'cache.t2.small',
            engine: 'redis',
            numCacheNodes: 1,
            autoMinorVersionUpgrade: true,
            cacheSubnetGroupName: subnetGroup.cacheSubnetGroupName,
            vpcSecurityGroupIds: [this.securityGroup.securityGroupId],
        });

        this.address = `${this.cluster.attrRedisEndpointAddress}:${this.cluster.attrRedisEndpointPort}`;

        this.cluster.addDependsOn(subnetGroup);
    }
}
