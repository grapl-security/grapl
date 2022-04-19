import { VizGraph, Link, VizNode } from "../../../types/CustomTypes";
import { calcNodeRiskPercentile } from "./nodeCalcs";
import { riskOutline } from "./nodeColoring";

export const findNode = (id: number, nodes: VizNode[]) => {
    for (const node of nodes || []) {
        if (node.id === id) {
            return node;
        }
    }
    return null;
};

export const calcLinkRisk = (link: Link, Graph: VizGraph) => {
    let sourceNode: any =
        findNode(link.source, Graph.nodes) ||
        findNode((link as any).source.name, Graph.nodes);

    let targetNode: any =
        findNode(link.target as any, Graph.nodes) ||
        findNode((link as any).target.name as any, Graph.nodes);

    const sourceRisk: number = sourceNode.risk_score || 0;
    const targetRisk: number = targetNode.risk_score || 0;

    if (!sourceNode || !targetNode) {
        console.error(
            "Missing srcNode/dstNode",
            sourceNode,
            link.source,
            targetNode,
            Graph.nodes
        );
        return 0;
    }
    return Math.round((sourceRisk + targetRisk) / 2);
};

export const calcLinkRiskPercentile = (link: Link, graph: VizGraph) => {
    const linkRisk = calcLinkRisk(link, graph);
    const nodes = [...graph.nodes].map((node) => node.risk);

    return calcNodeRiskPercentile(linkRisk, nodes);
};

export const calcLinkColor = (link: Link, graph: VizGraph): string => {
    const risk = calcLinkRiskPercentile(link, graph);
    return riskOutline(risk) as string;
};
