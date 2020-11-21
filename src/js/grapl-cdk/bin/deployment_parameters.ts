import * as ec2 from '@aws-cdk/aws-ec2';

module HardcodedDeploymentParameters {
    // ex: 'Grapl-my-deployment'
    export const deployName = 'jgrillo-test';

    // defaults to 'latest'
    export const graplVersion = 'jgrillo-test';

    // (optional) ex: ops@example.com
    export const watchfulEmail = 'jgrillo@graplsecurity.com';
    export const operationalAlarmsEmail = 'jgrillo@graplsecurity.com';
    export const securityAlarmsEmail = 'jgrillo@graplsecurity.com';

    // instance type for DGraph nodes
    export const dgraphInstanceType = undefined;

    export const region = 'us-east-1';
}

export module DeploymentParameters {
    const deployName = process.env.GRAPL_CDK_DEPLOYMENT_NAME
        || HardcodedDeploymentParameters.deployName;
    if (!deployName) {
        throw new Error('Error: Missing Grapl deployment name. Set via bin/deployment_parameters.ts, or environment variable GRAPL_CDK_DEPLOYMENT_NAME.');
    }
    export const stackName = deployName;

    export const graplVersion = process.env.GRAPL_VERSION
        || HardcodedDeploymentParameters.graplVersion
        || 'latest';

    export const watchfulEmail = process.env.GRAPL_CDK_WATCHFUL_EMAIL
        || HardcodedDeploymentParameters.watchfulEmail;

    export const operationalAlarmsEmail = process.env.GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL
        || HardcodedDeploymentParameters.operationalAlarmsEmail;

    export const securityAlarmsEmail = process.env.GRAPL_CDK_SECURITY_ALARMS_EMAIL
        || HardcodedDeploymentParameters.securityAlarmsEmail;

    const dgraphInstanceTypeName = process.env.GRAPL_DGRAPH_INSTANCE_TYPE
        || HardcodedDeploymentParameters.dgraphInstanceType
    export const dgraphInstanceType = new ec2.InstanceType(
        dgraphInstanceTypeName || 't3a.medium'
    );

    export const region = process.env.GRAPL_REGION
        || HardcodedDeploymentParameters.region
    if (!region) {
        throw new Error('Error: Missing Grapl region. Set via bin/deployment_parameters.ts or environment variable GRAPL_REGION.');
    }
}
