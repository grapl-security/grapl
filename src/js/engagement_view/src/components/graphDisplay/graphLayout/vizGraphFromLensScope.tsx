import { mapEdges, traverseNeighbors, traverseNodes } from "./graphTraverse";
import { getNodeLabel } from "./labels";
import {
    BaseNodeProperties,
    Link,
    Node,
    Risk,
    SummaryLink,
    VizGraph,
    VizNode,
} from "../../../types/CustomTypes";


export const getNodeType = (node: BaseNodeProperties) => {
    const dgraphType = node.dgraph_type;

    if (dgraphType) {
        if (Array.isArray(dgraphType)) {
            return dgraphType[0];
        }
        return dgraphType;
    }

    console.warn("Unable to find type for node ", node);
    return "Unknown Type";
};

export const summarizeLinks = (links: Link[]): SummaryLink[] => {
    const mergedLinks: Map<string, Link[]> = new Map();
    const newLinks: SummaryLink[] = [];

    for (const link of links) {
        const key = [link.source, link.target];
        key.sort();

        const sKey = String(key[0]) + String(key[1]);

        const names = mergedLinks.get(sKey) || [];

        names.push(link);

        mergedLinks.set(sKey, names);
    }

    // For now, we're effectively choosing the name and directionality at random. In a future PR we'll specify this behavior.
    for (const [key, innerLinks] of mergedLinks.entries()) {
        newLinks.push({
            source: innerLinks[0].source,
            target: innerLinks[0].target,
            name: innerLinks[0].name,
            innerLinks: innerLinks,
        });
    }

    return newLinks;
};
export const vizGraphFromLensScope = (vizGraphData: Node[]): VizGraph => {
    const nodes: VizNode[] = [];
    const links: Link[] = [];
    const vizNodeMap: Map<number, VizNode> = new Map();

    for (const vizNode of vizGraphData) {
        traverseNeighbors(vizNode, (fromNode, edgeName, toNode) => {
            if (edgeName !== "scope") {
                if (
                    getNodeType(fromNode) === "Unknown" ||
                    getNodeType(toNode) === "Unknown" ||
                    getNodeType(fromNode) === "Risk" ||
                    getNodeType(toNode) === "Risk"
                ) {
                    return;
                }

                links.push({
                    source: fromNode.uid,
                    name: edgeName,
                    target: toNode.uid,
                });
            }
        });

        traverseNodes(vizNode, (node) => {
            const nodeType = getNodeType(node);

            if (nodeType === "Unknown" || nodeType === "Risk") {
                return;
            }

            const nodeLabel = getNodeLabel(nodeType, node);
            const strippedNode = { ...node };

            let riskScore = (node["risk"] || 0) as number;
            let analyzerNames = "";
            let nodeRiskList = (node["risks"] || []) as Risk[];

            for (const riskNode of nodeRiskList) {
                riskScore += riskNode.risk_score || 0;

                if (analyzerNames && riskNode.analyzer_name) {
                    analyzerNames += ", ";
                }
                analyzerNames += riskNode.analyzer_name || "";
            }

            mapEdges(node, (edge: string, _neighbor: Node) => {
                // The stripped node is converted to another type, so we can cast to any here
                (strippedNode as any)[edge] = undefined;
            });

            const vizNode = {
                name: node.uid,
                ...strippedNode,
                risk_score: riskScore,
                analyzerNames,
                id: node.uid,
                nodeType,
                nodeLabel,
            };
            vizNodeMap.set(node.uid, (vizNode as unknown) as VizNode); // as unknown handles destructuring.
        });
    }

    // Because "nodes" is an array object we need to convert our data to use the
    // id property values of entries in the array instead of the indexes of the array.
    const index = {} as { [key: number]: VizNode };

    for (const vizNode of vizNodeMap.values()) {
        nodes.push(vizNode);
        if (!index[vizNode.uid]) {
            index[vizNode.uid] = vizNode;
        }
    }

    const summarizedLinks = summarizeLinks(links);
    // Return data in format for react-force-graph display
    return {
        nodes,
        links: summarizedLinks,
        index,
    };
};
