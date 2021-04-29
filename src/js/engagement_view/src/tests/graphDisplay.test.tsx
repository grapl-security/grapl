import {
    getNodeType,
    vizGraphFromLensScope,
} from "components/graphDisplay/graphLayout/vizGraphFromLensScope";

import { baseNodeData, processNode, rawNode, displayNodeData, hidden } from "./engagementView/data/baseData";

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

import {
    nodeFillColor,
    riskOutline
} from "components/graphDisplay/graphVizualization/nodeColoring";

import {mapNodeProps} from "components/graphDisplay/graphLayout/mapNodeProps"

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
    })

    expect(displayNode).toMatchObject(displayNodeData)
})

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

test("link label for children", () => {
    expect(getLinkLabel("children")).toBe("executed")
} )

// Test graph styling for high risk process node
// Assuming if process works, formatting for other node types also works
test("link label for processes", () => {
    expect(getLinkLabel("asset_processes")).toBe("asset_processes")
} )

test("get node label", () => {
    expect(getNodeLabel("Process", processNode as any)).toBe("Process")
})

test("get nodeFillColor for process", () => {
    expect(nodeFillColor("Process")).toBe("rgba(31, 120, 180, .8)")
} )

test("get high riskOutline", () => {
    expect(riskOutline(80)).toBe("#E50F14")
} )