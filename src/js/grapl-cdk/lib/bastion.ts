import * as cdk from '@aws-cdk/core';
import * as ec2 from '@aws-cdk/aws-ec2';

export class Bastion extends cdk.Construct {
    constructor(
        scope: cdk.Construct,
        id: string,
        bastion_props: ec2.BastionHostLinuxProps
    ) {
        super(scope, id);

        new ec2.BastionHostLinux(scope, id, bastion_props);
    }
}
