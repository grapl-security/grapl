import * as ec2 from '@aws-cdk/aws-ec2';

module HardcodedDeploymentParameters {
    // ex: 'Grapl-my-deployment'

    export const deployName = undefined;

    // defaults to 'latest'
    export const graplVersion = undefined;

    // (optional) ex: ops@example.com
    export const watchfulEmail = undefined;
    export const operationalAlarmsEmail = undefined;
    export const securityAlarmsEmail = undefined;

    // AWS EC2 instance type for DGraph nodes
    export const dgraphInstanceType = undefined;

    // AWS region for this Grapl deployment
    export const region = undefined;
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

    const _operationalAlarmsEmail = process.env.GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL
        || HardcodedDeploymentParameters.operationalAlarmsEmail;
    if (!_operationalAlarmsEmail) {
        throw new Error('Error: Missing operational alarms email. Set via bin/deployment_parameters.ts, or environment variable GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL.');
    }
    export const operationalAlarmsEmail = _operationalAlarmsEmail;

    const _securityAlarmsEmail = process.env.GRAPL_CDK_SECURITY_ALARMS_EMAIL
        || HardcodedDeploymentParameters.securityAlarmsEmail;
    if (!_securityAlarmsEmail) {
        throw new Error('Error: Missing security alarms email. Set via bin/deployment_parameters.ts, or environment variable GRAPL_CDK_SECURITY_ALARMS_EMAIL.');
    }
    export const securityAlarmsEmail = _securityAlarmsEmail;

    const dgraphInstanceTypeName = process.env.GRAPL_DGRAPH_INSTANCE_TYPE
        || HardcodedDeploymentParameters.dgraphInstanceType
    if (!dgraphInstanceTypeName) {
        throw new Error('Error: Missing Grapl DGraph instance type. Set via bin/deployment_parameters.ts or environment variable GRAPL_DGRAPH_INSTANCE_TYPE.');
    }
    export const dgraphInstanceType = new ec2.InstanceType(dgraphInstanceTypeName);

    export const region = process.env.GRAPL_REGION
        || HardcodedDeploymentParameters.region
    if (!region) {
        throw new Error('Error: Missing Grapl region. Set via bin/deployment_parameters.ts or environment variable GRAPL_REGION.');
    }
}
