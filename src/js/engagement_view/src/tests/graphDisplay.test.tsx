import {
    getNodeType,
    vizGraphFromLensScope,
} from "components/graphDisplay/graphLayout/vizGraphFromLensScope";
import { baseNodeData } from "./engagementView/data/baseNodeData";

import {
    vizGraphData,
    vizGraphReturnData,
} from "./engagementView/data/graphVizData";
import { mergeGraphs } from "components/graphDisplay/graphLayout/mergeGraphs";
import {
    initalGraphData,
    updatedGraphData,
} from "./engagementView/data/mergeGraphData";

// graphQLAdjacencyMatrix
test("get node type from dGraph type", () => {
    expect(getNodeType(baseNodeData)).toBe("Risk");
});

test("create graph from lens scope for graph display vizualization", () => {
    expect(vizGraphFromLensScope(vizGraphData as any)).toMatchObject(
        vizGraphReturnData
    );
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
