import { VizNode } from "../types/CustomTypes";
import { getNodeType, vizGraphFromLensScope } from "../components/graphDisplay/graphLayout/vizGraphFromLensScope"
import { baseNodeData } from "./engagementView/data/baseNodeData";
import {mergeNodes} from "../components/graphDisplay/graphLayout/mergeGraphs";

import {vizGraphData, vizGraphReturnData} from "./engagementView/data/graphVizData";
import { initialNodeX, initialNodeY } from "./engagementView/data/mergeNodeData";



// graphQLAdjacencyMatrix
test("get node type from dGraph type", () => {
	expect(getNodeType(baseNodeData)).toBe("Risk");
});

test("create graph from lens scope for graph display vizualization", () => {
	expect(vizGraphFromLensScope(vizGraphReturnData as any)).toMatchObject(vizGraphData ); 
})

test("nodes merge successfully", () => {
	expect(mergeNodes(initialNodeX as unknown as VizNode, initialNodeY as unknown as VizNode)).toBeTruthy();
})