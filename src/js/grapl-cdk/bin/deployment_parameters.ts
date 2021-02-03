module HardcodedDeploymentParameters {
    // ex: 'Grapl-my-deployment'
    export const deployName = undefined;

    // defaults to 'latest'
    export const graplVersion = undefined;

    // (optional) ex: ops@example.com
    export const watchfulEmail = undefined;
    export const operationalAlarmsEmail = undefined;
    export const securityAlarmsEmail = undefined;

    // AWS region for this Grapl deployment
    export const region = undefined;

    // (optional) The log level for services, ex: debug
    export const defaultLogLevel = undefined;
    // (optional) Override log levels for services
    export const sysmonSubgraphGeneratorLogLevel = undefined;
    export const osquerySubgraphGeneratorLogLevel = undefined;
    export const nodeIdentifierLogLevel = undefined;
    export const graphMergerLogLevel = undefined;
    export const analyzerDispatcherLogLevel = undefined;
    export const analyzerExecutorLogLevel = undefined;
    export const engagementCreatorLogLevel = undefined;
}

export module DeploymentParameters {
    const deployName = process.env.GRAPL_CDK_DEPLOYMENT_NAME
        || HardcodedDeploymentParameters.deployName;
    if (!deployName) {
        throw new Error('Error: Missing Grapl deployment name. Set via bin/deployment_parameters.ts, or environment variable GRAPL_CDK_DEPLOYMENT_NAME.');
    }
    export const stackName = deployName;
    export const defaultLogLevel = process.env.DEFAULT_LOG_LEVEL
        || HardcodedDeploymentParameters.defaultLogLevel
        || "INFO";

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

    export const region = process.env.GRAPL_REGION
        || HardcodedDeploymentParameters.region
    if (!region) {
        throw new Error('Error: Missing Grapl region. Set via bin/deployment_parameters.ts or environment variable GRAPL_REGION.');
    }

    export const sysmonSubgraphGeneratorLogLevel = process.env.SYSMON_SUBGRAPH_GENERATOR_LOG_LEVEL
        || HardcodedDeploymentParameters.sysmonSubgraphGeneratorLogLevel
        || defaultLogLevel;

    export const osquerySubgraphGeneratorLogLevel = process.env.OSQUERY_SUBGRAPH_GENERATOR_LOG_LEVEL
        || HardcodedDeploymentParameters.osquerySubgraphGeneratorLogLevel
        || defaultLogLevel;

    export const nodeIdentifierLogLevel = process.env.NODE_IDENTIFIER_LOG_LEVEL
        || HardcodedDeploymentParameters.nodeIdentifierLogLevel
        || defaultLogLevel;

    export const graphMergerLogLevel = process.env.GRAPH_MERGER_LOG_LEVEL
        || HardcodedDeploymentParameters.graphMergerLogLevel
        || defaultLogLevel;

    export const analyzerDispatcherLogLevel = process.env.ANALYZER_DISPATCHER_LOG_LEVEL
        || HardcodedDeploymentParameters.analyzerDispatcherLogLevel
        || defaultLogLevel;

    export const analyzerExecutorLogLevel = process.env.ANALYZER_EXECUTOR_LOG_LEVEL
        || HardcodedDeploymentParameters.analyzerExecutorLogLevel
        || defaultLogLevel;

    export const engagementCreatorLogLevel = process.env.ENGAGEMENT_CREATOR_LOG_LEVEL
        || HardcodedDeploymentParameters.engagementCreatorLogLevel
        || defaultLogLevel;

}
