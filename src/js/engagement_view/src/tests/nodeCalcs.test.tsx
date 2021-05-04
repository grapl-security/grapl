import { vizGraphReturnData } from "./engagementView/testData/graphVizData";

import {
    calcNodeRiskPercentile,
    nodeSize,
} from "components/graphDisplay/graphVizualization/nodeCalcs";

test("nodeSize calculation", () => {
    const node = vizGraphReturnData.nodes[0];
    expect(nodeSize(node, vizGraphReturnData)).toBe(7);
});

test("calcNodeRiskPercentile calculation", () => {
    const risks = [100, 59, 50, 85, 100, 13];
    expect(calcNodeRiskPercentile(50, risks)).toBe(33);
});
