import {
    getNodeType,
    vizGraphFromLensScope,
} from "components/graphDisplay/graphLayout/vizGraphFromLensScope";

import { baseNodeData, processNode } from "./engagementView/data/baseData";

import {
    vizGraphData,
    vizGraphReturnData,
} from "./engagementView/data/graphVizData";

import { mergeGraphs } from "components/graphDisplay/graphLayout/mergeGraphs";

import {
    initalGraphData,
    updatedGraphData,
} from "./engagementView/data/mergeGraphData";

import {
    getLinkLabel,
    getNodeLabel,
} from "components/graphDisplay/graphLayout/labels";

// graphQLAdjacencyMatrix
test("get node type from dGraph type", () => {
    expect(getNodeType(baseNodeData)).toBe("Risk");
});

test("merge graph data HAS changed and graph WILL be updated", () => {
    expect(
        mergeGraphs(initalGraphData as any, updatedGraphData as any)
    ).toMatchObject(updatedGraphData);
});

test("merge graph data has NOT changed and graph WILL NOT be updated", () => {
    expect(mergeGraphs(initalGraphData as any, initalGraphData as any)).toBe(
        null
    );
});

test("link label for children", () => {
    expect(getLinkLabel("children")).toBe("executed")
} )

test("link label for processes", () => {
    expect(getLinkLabel("asset_processes")).toBe("asset_processes")
} )

test("get node label", () => {
    expect(getNodeLabel("Process", processNode as any)).toBe("Process")
})