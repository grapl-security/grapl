import {
    vizGraphReturnData,
    node40,
} from "./engagementView/testData/graphVizData";

import {
    findNode,
    calcLinkColor,
    calcLinkRisk,
} from "components/graphDisplay/graphVizualization/linkCalcs";

test("findNode", () => {
    const nodes = vizGraphReturnData.nodes;
    expect(findNode(40, nodes)).toMatchObject(node40);
});

test("calcLinkColor", () => {
    const link = vizGraphReturnData.links[0];
    expect(calcLinkColor(link, vizGraphReturnData as any)).toBe("#DA634F");
});

test("calcLinkRisk", () => {
    const link = vizGraphReturnData.links[0];
    expect(calcLinkRisk(link, vizGraphReturnData as any)).toBe(48);
});
