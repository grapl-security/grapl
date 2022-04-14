import {
    baseNodeData,
    processNode,
    rawNode,
    displayNodeData,
    hidden,
} from "./engagementView/testData/baseData";

import {
    vizGraphData,
    vizGraphReturnData,
} from "./engagementView/testData/graphVizData";

import {
    initalGraphData,
    updatedGraphData,
} from "./engagementView/testData/mergeGraphData";

import {
    getNodeType,
    vizGraphFromLensScope,
} from "components/graphDisplay/graphLayout/vizGraphFromLensScope";

import { mergeGraphs } from "components/graphDisplay/graphLayout/mergeGraphs";

import {
    nodeFillColor,
    riskOutline,
} from "components/graphDisplay/graphVizualization/nodeColoring";

import { mapNodeProps } from "components/graphDisplay/graphLayout/mapNodeProps";

test("map display node properties for node table", () => {
    const displayNode = {} as any;

    mapNodeProps(rawNode as any, (propName: string) => {
        const prop = rawNode[propName];

        if (!hidden.has(propName)) {
            if (prop) {
                if (propName.includes("_time")) {
                    try {
                        displayNode[propName] = new Date(prop).toLocaleString();
                    } catch (e) {
                        displayNode[propName] = prop;
                    }
                } else {
                    displayNode[propName] = prop;
                }
            }
        }
    });

    expect(displayNode).toMatchObject(displayNodeData);
});

// graphQLAdjacencyMatrix
test("get node type from dGraph type", () => {
    expect(getNodeType(baseNodeData)).toBe("Risk");
});

test("create graph from lens scope for graph display vizualization for react-force-graph", () => {
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

test("get nodeFillColor for process", () => {
    expect(nodeFillColor("Process")).toBe("rgba(31, 120, 180, .8)");
});

test("get high riskOutline", () => {
    expect(riskOutline(80)).toBe("#DA634F");
});
