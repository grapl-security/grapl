import { calcNodeRiskPercentile } from '../calculations/node/nodeCalcs';
import { VizNode, VizGraph } from '../../../../types/CustomTypes';

export const nodeSize = (node: VizNode, Graph: VizGraph) => {
    const nodes = [...Graph.nodes].map(node => node.risk);
    const riskPercentile = calcNodeRiskPercentile(node.risk, nodes);

    if (riskPercentile >= 75) {
        return 6
    } else if (riskPercentile >= 25) {
        return 5
    } else {
        return 4
    }
};
