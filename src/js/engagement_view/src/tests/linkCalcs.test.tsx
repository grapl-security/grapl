import {
    vizGraphReturnData,
    node40,
} from "./engagementView/testData/graphVizData";

import {
    findNode,
    calcLinkDirectionalArrowRelPos,
    calcLinkParticleWidth,
    calcLinkColor,
    calcLinkRisk,
} from "components/graphDisplay/graphVizualization/linkCalcs";

test("findNode", () => {
    const nodes = vizGraphReturnData.nodes;
    expect(findNode(40, nodes)).toMatchObject(node40);
});

test("calcLinkDirectionalArrowRelPos", () => {
    const link = vizGraphReturnData.links[0];
    expect(
        calcLinkDirectionalArrowRelPos(link, vizGraphReturnData as any)
    ).toEqual(1);
});

test("calcLinkParticleWidth", () => {
    const link = vizGraphReturnData.links[0];
    expect(calcLinkParticleWidth(link, vizGraphReturnData as any)).toEqual(5);
});

test("calcLinkColor", () => {
    const link = vizGraphReturnData.links[0];
    expect(calcLinkColor(link, vizGraphReturnData as any)).toBe("#E50F14");
});

test("calcLinkRisk", () => {
    const link = vizGraphReturnData.links[0];
    expect(calcLinkRisk(link, vizGraphReturnData as any)).toBe(48);
});
